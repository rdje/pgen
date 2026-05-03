# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.1.33 / Contract 1.1.35** (slice 2 of the typed-shape campaign).
- Slices 3 and 4 (typed `counted_quantifier_body` + `null` literal, then typed `counted_quantifier`) are landed on main but the consolidated contract identity bump for them lands together with the next quantifier-subtree slice that closes the outer `quantifier` rule.
- Until then, the contract document still shows `1.1.33 / 1.1.35` while this book describes the post-slice-3+4 shape that's actually emitted by main HEAD.

Below are the shape-change highlights of recent slices, with pointers to the contract sections (where applicable).

### 1.1.65 / Contract 1.1.67 — Slice 35: directive_payload_suffix typed + directive_payload_simple regex-literal rewrite

**What changed:** `directive_payload_suffix` per-branch typed; `directive_payload_simple` rewritten as regex literal.

```ebnf
directive_payload_suffix = ":" directive_payload_simple?  -> {separator: ":", value: $2}
                         | "=" directive_payload_simple?  -> {separator: "=", value: $2}
directive_payload_simple = /([^)]*)/  # was directive_payload_char* chain
```

**Before / after (visible inside `directive_verb.body.payload`):**

| Source | Before (slice 34) | After |
|---|---|---|
| `(*MARK:foo)` | `payload:[":", ["f","o","o"]]` | `payload:{separator:":", value:"foo"}` |
| `(*COMMIT)` | `payload:[]` (un-matched) | `payload:[]` (unchanged — optional slot) |
| `(*:short)` (mark_shorthand) | `payload:["s","h","o","r","t"]` | `payload:"short"` (clean string) |

**`directive_verb.body` now end-to-end typed** across all 3 sub-rule levels (body / named\|shorthand / payload).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.65 / Contract 1.1.67 Highlights".

### 1.1.64 / Contract 1.1.66 — Slice 34: directive_body / directive_named / directive_mark_shorthand typed + directive_name regex-literal rewrite

**What changed:** `directive_body`'s 2 sub-rules typed; `directive_name` rewritten as regex literal for clean string output.

```ebnf
directive_named          = directive_name directive_payload_suffix?  -> {kind: "named", name: $1, payload: $2}
directive_mark_shorthand = ":" directive_payload_simple?             -> {kind: "mark_shorthand", payload: $2}
directive_name           = /([A-Za-z][A-Za-z0-9_\-]*)/  # was directive_name_start directive_name_continue*
```

**Before / after (visible inside `directive_verb.body`):**

| Source | Before (slice 24) | After |
|---|---|---|
| `(*MARK:foo)` | `body:[["M", ["A","R","K"]], [":", [...]]]` (raw chain) | `body:{kind:"named", name:"MARK", payload:[":", ["f","o","o"]]}` |
| `(*COMMIT)` | similar raw chain | `body:{kind:"named", name:"COMMIT", payload:[]}` |
| `(*:bar)` | similar raw chain | `body:{kind:"mark_shorthand", payload:["b","a","r"]}` |

**`name` is now a clean string** (slice 34's regex-literal rewrite — same pattern as `name`/`hex_digits`/`octal_digits`/`prop_name`/`comment_text` in earlier slices).

**`payload` carries raw shape.** Per-rule typing of `directive_payload_suffix` / `directive_payload_simple` is a separate concern.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.64 / Contract 1.1.66 Highlights".

### 1.1.63 / Contract 1.1.65 — Slice 33: callout_string typed (8 quote-form variants)

**What changed:** All 8 callout-string quote variants now emit typed `{quote:<text-label>, payload:<string>}` objects.

```ebnf
callout_backtick_string  = '`' callout_backtick_payload '`'    -> {quote: "backtick", payload: $2}
callout_single_string    = "'" callout_single_payload "'"      -> {quote: "single",   payload: $2}
callout_double_string    = '"' callout_double_payload '"'      -> {quote: "double",   payload: $2}
callout_caret_string     = "^" callout_caret_payload "^"       -> {quote: "caret",    payload: $2}
callout_percent_string   = "%" callout_percent_payload "%"     -> {quote: "percent",  payload: $2}
callout_hash_string      = "#" callout_hash_payload "#"        -> {quote: "hash",     payload: $2}
callout_dollar_string    = "$" callout_dollar_payload "$"      -> {quote: "dollar",   payload: $2}
callout_brace_string     = "{" callout_brace_payload "}"       -> {quote: "brace",    payload: $2}
```

**Before / after (visible inside `callout.arg`):**

| Source | Before (slice 24) | After |
|---|---|---|
| `` (?C`hello`) `` | `arg:["` `", "hello", "` `"]` | `arg:{quote:"backtick", payload:"hello"}` |
| `(?C'world')` | `arg:["'", "world", "'"]` | `arg:{quote:"single", payload:"world"}` |
| `(?C"dq")` | `arg:["\"", "dq", "\""]` | `arg:{quote:"double", payload:"dq"}` |
| `(?C{brace})` | `arg:["{", "brace", "}"]` | `arg:{quote:"brace", payload:"brace"}` |

**`quote` is a text label, not the literal character.** PGEN's bootstrap annotation parser doesn't support `"\""` (escaped double-quote) inside string literals — switched all 8 forms to text labels for uniformity. Consumer reads `arg.quote` as an enum-like discriminator: `"backtick"`/`"single"`/`"double"`/`"caret"`/`"percent"`/`"hash"`/`"dollar"`/`"brace"`.

**Numeric callout** (`(?C42)`) continues to surface `arg:42` (typed int from slice 1's digits @transform). Consumer dispatches: object → string callout; number → numeric callout; `[]` → empty `(?C)`.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.63 / Contract 1.1.65 Highlights".

### 1.1.62 / Contract 1.1.64 — Slice 32: define_condition / version_condition / recursion_condition typed

**What changed:** 4 annotations across 3 condition Or-alternatives — `define_condition`, `version_condition`, and `recursion_condition` (2 branches).

```ebnf
define_condition  = "DEFINE"                                          -> {kind: "define"}
version_condition = "VERSION" version_operator version_number         -> {kind: "version", operator: $2, number: $3}
recursion_condition = "R" digits?                                     -> {kind: "recursion", group: $2}
                    | "R&" name                                       -> {kind: "recursion_named", name: $2}
```

**Before / after (visible inside `conditional.condition`):**

| Source | Before (slice 27) | After |
|---|---|---|
| `(?(DEFINE)foo)` | `condition:"DEFINE"` (string — ambiguous w/ name) | `condition:{kind:"define"}` |
| `(?(VERSION>=10.0)foo)` | 3-element raw seq | `condition:{kind:"version", operator:">=", number:[10, [".", 0]]}` |
| `(?(R)bar)` | `condition:["R", []]` | `condition:{kind:"recursion", group:[]}` |
| `(?(R3)baz)` | `condition:["R", 3]` | `condition:{kind:"recursion", group:3}` |
| `(?(R&name)abc)` | `condition:["R&", "name"]` | `condition:{kind:"recursion_named", name:"name"}` |

**6 of 9 condition Or-alternatives now disambiguated.** The remaining 6 (`condition_callout_assertion`, `condition_assertion`, `name_ref`, `name`, `signed_digits`, `digits`) are reused outside the condition context and weren't wrapped to avoid changing their shape across all callers. Consumer dispatches by:
- Object with `kind`: typed condition.
- Object with `sign`/`value`: `signed_digits`.
- Number: `digits`.
- String: `name`/`name_ref`. (Now disambiguated from `(?(DEFINE)...)`.)
- Other: `condition_callout_assertion` / `condition_assertion` (raw, not yet typed).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.62 / Contract 1.1.64 Highlights".

### 1.1.61 / Contract 1.1.63 — Slice 31: modifier_spec typed

**What changed:** `modifier_spec` per-branch typed `{reset:<bool>, seq:<raw>}`.

```ebnf
modifier_spec = "^" modifier_seq?    -> {reset: true, seq: $2}
              | modifier_seq         -> {reset: false, seq: $1}
```

**Before / after (visible inside `inline_modifiers.spec` / `scoped_inline_modifiers.spec`):**

| Source | Before (slice 24) | After |
|---|---|---|
| `(?i)` | `spec:[[...], []]` | `spec:{reset:false, seq:[["i"], []]}` |
| `(?^i)` | similar w/ `"^"` token | `spec:{reset:true, seq:[["i"], []]}` |
| `(?ix-m)` | similar | `spec:{reset:false, seq:[["i", ["x", []]], ["-", ["m"]]]}` |
| `(?-i)` | similar | `spec:{reset:false, seq:["-", ["i"]]}` |

**`reset:true` distinguishes the `(?^...)` form** (which resets all flags first before applying the seq).

**`seq` carries the raw `modifier_seq` shape.** Per-rule typing of `modifier_seq` / `modifier_group` / `modifier_item` (which would unify the flag-set into a clean `{set:["i", "x"], unset:["m"]}` shape) is a separate concern.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.61 / Contract 1.1.63 Highlights".

### 1.1.60 / Contract 1.1.62 — Slice 30: subroutine_target typed

**What changed:** `subroutine_target` now emits typed `{kind, ...}` objects per branch.

```ebnf
subroutine_target = "&" name           -> {kind: "named", name: $2}
                  | "P>" name          -> {kind: "python_named", name: $2}
                  | "R"                -> {kind: "recursion"}
                  | signed_digits      -> {kind: "numeric", value: $1.value, sign: $1.sign}
```

**Before / after (visible inside `subroutine_call.target`):**

| Source | Before (slice 25) | After |
|---|---|---|
| `(?&name)` | `target:["&", "name"]` | `target:{kind:"named", name:"name"}` |
| `(?P>foo)` | `target:["P>", "foo"]` | `target:{kind:"python_named", name:"foo"}` |
| `(?R)` | `target:"R"` | `target:{kind:"recursion"}` |
| `(?+1)` | `target:{sign:"+", value:1}` | `target:{kind:"numeric", sign:"+", value:1}` |
| `(?-2)` | similar | `target:{kind:"numeric", sign:"-", value:2}` |
| `(?42)` | `target:{sign:[], value:42}` | `target:{kind:"numeric", sign:[], value:42}` |

**`subroutine_call.target` now end-to-end typed.** All 4 syntactic forms surface as `{kind, ...}` objects with consistent dispatch.

**`kind:"python_named"` vs `kind:"named"`** — preserves Python syntax origin, paralleling slice 19's `python_named_backreference` and slice 22's `python_named_group`. Consumers normalizing across name-based forms: `target.kind in {"named", "python_named"}` → name-based subroutine; `target.name` carries the name string in both.

**Numeric form: `signed_digits` field-access inline.** The annotation `{kind:"numeric", value:$1.value, sign:$1.sign}` uses the field-access syntax to inline signed_digits' `{sign, value}` typed shape (slice 13) into subroutine_target's typed shape. Consumer reads `target.value` / `target.sign` directly without an extra nested object.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.60 / Contract 1.1.62 Highlights".

### 1.1.59 / Contract 1.1.61 — Slice 29: class_range / quoted_class_literal / class_range_escape typed

**What changed:** First sub-rule typing slice — the inner shapes that show up under `char_class.body`.

```ebnf
class_range          = class_atom class_zero_width* "-" class_zero_width* class_atom
                        -> {type: "class_range", start: $1, end: $5}
quoted_class_literal = "\\Q" quoted_class_literal_char* "\\E"
                        -> {type: "class_quoted_literal", body: $2}
class_range_escape   = "\\" class_range_escape_unit
                        -> $2
```

**Before / after:**

| Source | Before (slice 28) | After |
|---|---|---|
| `[a-z]` body | `[["a", [], "-", [], "z"]]` | `[{type:"class_range", start:"a", end:"z"}]` |
| `[A-Z0-9_]` body | mix of class_range and class_literal | `[{type:"class_range", start:"A", end:"Z"}, {type:"class_range", start:"0", end:"9"}, "_"]` |
| `[\Qa-z\E]` body | `[["\\Q", ["a", "-", "z"], "\\E"]]` | `[{type:"class_quoted_literal", body:["a", "-", "z"]}]` |
| `[\xA-\xFF]` body | deeply nested | `[{type:"class_range", start:{type:"escape", kind:"hex", digits:"A"}, end:{type:"escape", kind:"hex", digits:"FF"}}]` |

**`class_range_escape -> $2` is a transparent passthrough.** Drops the leading `\` so the typed escape_unit shape (already produced by hex/unicode/octal/control/simple/single_byte/property branches via slices 14-17) surfaces directly inside `class_range.start` / `class_range.end`. Mirrors the outer `escape -> $2` annotation from slice 14.

**Char_class body shapes now typed end-to-end.** Plain literals → bare string (slice 15's class_literal regex literal); ranges → `{type:"class_range"}`; quoted literals → `{type:"class_quoted_literal"}`; escapes → typed escape_unit shapes (slices 14-17); POSIX classes → typed `posix_class` (slice 8).

**Sub-rule typing campaign progress:** First slice. Atom subtree stays 25/25 at outer level; sub-rule typing now covers the most-visible inner shapes inside `char_class.body`.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.59 / Contract 1.1.61 Highlights".

### 1.1.58 / Contract 1.1.60 — Atom subtree slice 28: extended_class typed (recursive structures all closed)

**What changed:** `extended_class` now emits typed `{type:"atom", kind:"extended_class", body:<content>}` objects.

```ebnf
extended_class = "(?[" extended_class_content "])"
                  -> {type: "atom", kind: "extended_class", body: $2}
```

**Before / after:**

| Source | After |
|---|---|
| `(?[abc])` | `{kind:"extended_class", body:["a","b","c"]}` |
| `(?[a-z])` | `{body:["a","-","z"]}` |
| `(?[[abc][def]])` | `{body:[["[", ["a","b","c"], "]"], ["[", ["d","e","f"], "]"]]}` (nested) |
| `(?[])` | `{body:[]}` |

`body` is the raw `extended_class_content` shape (Quantified-* of extended_class_element). Sub-rule typing of content/element/nested (the recursive set-operation structure: `[X]&[Y]`, `[X]+[Y]`, etc.) is a separate concern.

**Atom subtree campaign progress: 25/25 atom alternatives directly typed.** All recursive structures closed. Only the 3 deferred leaf-char alternatives (literal / whitespace_literal / dot) remain as a separate decision.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.58 / Contract 1.1.60 Highlights".

### 1.1.57 / Contract 1.1.59 — Atom subtree slice 27: conditional typed

**What changed:** `conditional` now emits typed `{type:"atom", kind:"conditional", condition, yes_branch, no_branch}` objects.

```ebnf
conditional        = "(?(" condition ")" yes_branch ("|" no_branch)? ")"
                       -> {type: "atom", kind: "conditional", condition: $2, yes_branch: $4, no_branch: $5}
conditional_branch = piece*    -> [$1**]
```

**Before / after:**

| Source | After |
|---|---|
| `(?(1)abc)` | `{kind:"conditional", condition:{sign:[], value:1}, yes_branch:[<3 pieces>], no_branch:[]}` |
| `(?(1)abc|xyz)` | `{condition:{sign:[], value:1}, yes_branch:<3 pieces>, no_branch:["|", <3 pieces>]}` |
| `(?(DEFINE)foo)` | `{condition:"DEFINE", yes_branch:<3 pieces>, no_branch:[]}` |
| `(?(R)bar)` | `{condition:["R", []], yes_branch:<3 pieces>, no_branch:[]}` |
| `(?(<name>)abc)` | `{condition:"name", yes_branch:<3 pieces>, no_branch:[]}` |

**`condition` is the heterogeneous Or-of-9 raw shape.** Typed signed_digits propagation (slice 13) gives `{sign, value}` for numeric refs; `"DEFINE"` string for `(?(DEFINE)...)`; `["R", []]` for recursion conditions; clean `name` string for named-group refs. Sub-rule typing of `condition` is a separate concern.

**`no_branch` preserves the `|` separator.** `[]` when no else-clause; `["|", <pieces>]` when matched. Consumer reads `no_branch[1]` to extract pieces or maps `[]` to null. Distinguishing "no else-clause" from "empty else-clause" is intentional.

**`conditional_branch` flat-piece-array** (`[$1**]`) parallels `concatenation`'s shape — consumer iterates directly.

**Atom subtree campaign progress:** 24/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.57 / Contract 1.1.59 Highlights".

### 1.1.56 / Contract 1.1.58 — Atom subtree slice 26: char_class outer typed

**What changed:** `char_class` now emits typed `{type:"atom", kind:"char_class", negated:<bool>, initial_close:<bool>, body:<class_body>}` objects.

```ebnf
char_class          = "[" negation? class_initial_close? class_body "]"
                       -> {type: "atom", kind: "char_class", negated: $2, initial_close: $3, body: $4}
class_initial_close = "]"                                            -> true
negation            = "^"                                            -> true
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `[abc]` | `["[", [], [], <body>, "]"]` | `{type:"atom", kind:"char_class", negated:[], initial_close:[], body:["a","b","c"]}` |
| `[^abc]` | similar w/ `negation` shape | `{kind:"char_class", negated:true, body:["a","b","c"]}` |
| `[a-z]` | similar w/ `class_range` body | `{kind:"char_class", body:[["a", [], "-", [], "z"]]}` |
| `[]abc]` | similar w/ `class_initial_close` | `{kind:"char_class", initial_close:true, body:["a","b","c"]}` |
| `[^]abc]` | both | `{negated:true, initial_close:true, body:["a","b","c"]}` |
| `[[:alpha:]]` | typed posix_class inside | `{kind:"char_class", body:[{type:"posix_class", name:"alpha", negated:[]}]}` |

**`negated` and `initial_close` are real booleans** (`true` matched, `[]` un-matched). Same convention as `posix_negation` from slice 8 (PGEN-RGX-0076) — `BooleanLiteral` rule-level scalar emits `Json(Bool(true))`.

**`body` is raw `class_body` shape.** Inner items already typed by earlier slices propagate transparently — `posix_class` (slice 8), `class_range_escape` (escape subtree slices), `quoted_class_range_atom` (PGEN-RGX-0068 fix). Pure char-class typing is end-to-end at the outer level; `class_body` per-rule typing is a separate concern.

**Atom subtree campaign progress:** 23/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.56 / Contract 1.1.58 Highlights".

### 1.1.55 / Contract 1.1.57 — Atom subtree slice 25: scan_substring / script_run / subroutine_call typed

**What changed:** Batched slice — 4 annotations across 3 atom alternatives.

```ebnf
scan_substring_group = "(*" scan_substring_name ":" returned_capture_group_list pattern? ")"
                        -> {type: "atom", kind: "scan_substring_group", name: $2, captures: $4, body: $5}
script_run_group     = "(*" script_run_name ":" pattern? ")"
                        -> {type: "atom", kind: "script_run_group", name: $2, body: $4}
subroutine_call      = "(?" returned_capture_subroutine ")"
                        -> {type: "atom", kind: "subroutine_call", target: $2}
                     | "(?" subroutine_target ")"
                        -> {type: "atom", kind: "subroutine_call", target: $2}
```

**Before / after:**

| Source | After |
|---|---|
| `(*sr:abc)` | `{kind:"script_run_group", name:"sr", body:<pattern>}` |
| `(*atomic_script_run:foo)` | `{kind:"script_run_group", name:"atomic_script_run", body:<pattern>}` |
| `(a)(*scs:(1)bcd)` | `{kind:"scan_substring_group", name:"scs", captures:["(", {sign:[], value:1}, [], ")"], body:<pattern>}` |
| `(?&name)` | `{kind:"subroutine_call", target:["&", "name"]}` |
| `(?P>name)` | `{kind:"subroutine_call", target:["P>", "name"]}` |
| `(?R)` | `{kind:"subroutine_call", target:"R"}` |
| `(?+1)` | `{kind:"subroutine_call", target:{sign:"+", value:1}}` |

**`subroutine_call` two-branch collapse:** Both branches produce `kind:"subroutine_call"` with `target` carrying the inner shape. Branch 0 is the "with returned captures" form (`returned_capture_subroutine`); branch 1 is the plain form. Consumers inspect `target` shape to determine which form.

**Sub-rule shapes deferred:** `returned_capture_group_list`, `returned_capture_subroutine`, `subroutine_target` carry raw shapes. Per-rule typing is a separate concern.

**`signed_digits` propagation already works:** Slice 13's `signed_digits -> {sign, value}` typing surfaces directly inside `subroutine_call.target` for numeric subroutine refs like `(?+1)`. This wasn't a slice 25 change but is the visible payoff of the earlier work.

**Pre-existing host-validator note:** PGEN's host-side compile validator rejects scan_substring capture-list references that don't have a corresponding group in the surrounding pattern. The annotation is correct when the validator allows; for inputs the validator rejects, no AST is produced.

**Atom subtree campaign progress:** 22/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.55 / Contract 1.1.57 Highlights".

### 1.1.54 / Contract 1.1.56 — Atom subtree slice 24: inline-modifier / callout / directive_verb / code_block typed

**What changed:** Batched slice — 6 annotations across 5 atom alternatives.

```ebnf
inline_modifiers        = "(?" modifier_spec? ")"            -> {type: "atom", kind: "inline_modifiers", spec: $2}
scoped_inline_modifiers = "(?" modifier_spec ":" pattern? ")" -> {type: "atom", kind: "scoped_inline_modifiers", spec: $2, body: $4}
callout                 = "(?C" callout_arg? ")"             -> {type: "atom", kind: "callout", arg: $2}
directive_verb          = "(*" directive_body ")"            -> {type: "atom", kind: "directive_verb", body: $2}
code_block_plain        = "(?{" code_content "})"            -> {type: "atom", kind: "code_block", lang: null, content: $2}
code_block_lang         = "(?{" code_lang ":" ws? code_content "})"
                                                              -> {type: "atom", kind: "code_block", lang: $2, content: $4}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?i)` | `["(?", [<modifier_spec>], ")"]` | `{kind:"inline_modifiers", spec:<modifier_spec>}` |
| `(?i:abc)` | `["(?", <modifier>, ":", <pattern>, ")"]` | `{kind:"scoped_inline_modifiers", spec:<modifier>, body:<pattern>}` |
| `(?C42)` | `["(?C", 42, ")"]` | `{kind:"callout", arg:42}` |
| `(*MARK:foo)` | `["(*", <directive>, ")"]` | `{kind:"directive_verb", body:<directive>}` |
| `(?{print})` | `["(?{", <content>, "})"]` | `{kind:"code_block", lang:null, content:<content>}` |
| `(?{lua: print})` | `["(?{", "lua", ":", <ws?>, <content>, "})"]` | `{kind:"code_block", lang:"lua", content:<content>}` |

**`code_block` two-branch collapse:** Both `code_block_plain` and `code_block_lang` produce `kind:"code_block"`, distinguished by `lang` (null vs string). Consumer always reads `obj.lang` and `obj.content` — no need to dispatch on which branch matched.

**Sub-rule shapes deferred:** `modifier_spec`, `callout_arg`, `directive_body`, `code_content` carry raw shapes. Their per-rule typing is left to follow-up slices. Atom-level dispatch on `kind` is what slice 24 delivers.

**Atom subtree campaign progress:** 19/25 atom alternatives directly typed (code_block counted as 1 atom alternative; its 2 branches collapse).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.54 / Contract 1.1.56 Highlights".

### 1.1.53 / Contract 1.1.55 — Atom subtree slice 23: lookaround family typed (7 sub-rules)

**What changed:** All 7 lookaround sub-rules now emit typed `{type:"atom", kind:<lookaround_kind>, ..., body:<pattern>}` objects.

```ebnf
lookahead_pos             = "(?=" pattern ")"            -> {type: "atom", kind: "lookahead",   positive: true,  body: $2}
lookahead_neg             = "(?!" pattern ")"            -> {type: "atom", kind: "lookahead",   positive: false, body: $2}
lookbehind_pos            = "(?<=" pattern ")"           -> {type: "atom", kind: "lookbehind",  positive: true,  body: $2}
lookbehind_neg            = "(?<!" pattern ")"           -> {type: "atom", kind: "lookbehind",  positive: false, body: $2}
non_atomic_lookahead_pos  = "(?*" pattern ")"            -> {type: "atom", kind: "non_atomic_lookahead",  positive: true, body: $2}
non_atomic_lookbehind_pos = "(?<*" pattern ")"           -> {type: "atom", kind: "non_atomic_lookbehind", positive: true, body: $2}
alpha_lookaround          = "(*" alpha_lookaround_name ":" pattern? ")"
                                                          -> {type: "atom", kind: "alpha_lookaround", name: $2, body: $4}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?=foo)` | `["(?=", <pattern>, ")"]` | `{type:"atom", kind:"lookahead", positive:true, body:<pattern>}` |
| `(?!bar)` | `["(?!", <pattern>, ")"]` | `{kind:"lookahead", positive:false, body:<pattern>}` |
| `(?<=baz)` | `["(?<=", <pattern>, ")"]` | `{kind:"lookbehind", positive:true, body:<pattern>}` |
| `(?<!qux)` | `["(?<!", <pattern>, ")"]` | `{kind:"lookbehind", positive:false, body:<pattern>}` |
| `(?*alpha)` | `["(?*", <pattern>, ")"]` | `{kind:"non_atomic_lookahead", positive:true, body:<pattern>}` |
| `(?<*beta)` | `["(?<*", <pattern>, ")"]` | `{kind:"non_atomic_lookbehind", positive:true, body:<pattern>}` |
| `(*pla:gamma)` | `["(*", "pla", ":", <pattern>, ")"]` | `{kind:"alpha_lookaround", name:"pla", body:<pattern>}` |
| `(*nla:delta)` | similar | `{kind:"alpha_lookaround", name:"nla", body:<pattern>}` |

**`kind` + `positive` design:** Lookahead and lookbehind each collapse 2 syntactic forms (`_pos`/`_neg`) to one `kind` with a `positive` boolean — consistent with the property_escape `negated` field convention from slice 17. Non-atomic forms get distinct `kind` values since PCRE2 only supports positive variants for them.

**Alpha-form** carries the alpha_lookaround_name in `name`. PCRE2 admits `pla`/`positive_lookahead`, `nla`/`negative_lookahead`, `plb`/`positive_lookbehind`, `nlb`/`negative_lookbehind`, `napla`/`non_atomic_positive_lookahead`, `naplb`/`non_atomic_positive_lookbehind`. Consumers map by `name` to dispatch on the semantic equivalent.

**Atom subtree campaign progress:** 14/25 atom alternatives directly typed. Lookaround family typed end-to-end.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.53 / Contract 1.1.55 Highlights".

### 1.1.52 / Contract 1.1.54 — Atom subtree slice 22: named groups typed (named/python_named)

**What changed:** `named_group` (both angle and quote syntactic forms) and `python_named_group` now emit typed `{type:"atom", kind:<group_kind>, name:<string>, body:<pattern>}` objects.

```ebnf
named_group        = "(?<" name ">" pattern? ")"  -> {type: "atom", kind: "named_group", name: $2, body: $4}
                   | "(?'" name "'" pattern? ")"  -> {type: "atom", kind: "named_group", name: $2, body: $4}
python_named_group = "(?P<" name ">" pattern? ")" -> {type: "atom", kind: "python_named_group", name: $2, body: $4}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?<foo>abc)` | `["(?<", "foo", ">", <pattern>, ")"]` | `{type:"atom", kind:"named_group", name:"foo", body:<pattern>}` |
| `(?'bar'xyz)` | `["(?'", "bar", "'", <pattern>, ")"]` | `{kind:"named_group", name:"bar", body:<pattern>}` |
| `(?P<baz>123)` | `["(?P<", "baz", ">", <pattern>, ")"]` | `{kind:"python_named_group", name:"baz", body:<pattern>}` |
| `(?<empty>)` | similar 5-element seq | `{kind:"named_group", name:"empty", body:[[], []]}` |

`name` was already typed to a clean string by slice 11. `body` is the raw pattern shape (pattern outer typing is a separate slice).

**`kind:"python_named_group"` distinct from `kind:"named_group"`.** Paralleling slice 19's `python_named_backreference` decision: PCRE2 treats `(?P<n>...)` and `(?<n>...)` as functionally equivalent, but tooling that displays the source pattern wants to preserve the syntactic origin. Consumers normalizing across name-based group forms: `kind in {"named_group", "python_named_group"}` → name-based group; `name` carries the name in both.

**Group typing now end-to-end.** All 6 group sub-rules typed:
- `capturing_group`, `noncapturing_group`, `named_group`, `python_named_group` (under `group`)
- `branch_reset_group`, `atomic_group` (standalone)

**Atom subtree campaign progress:** 13/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.52 / Contract 1.1.54 Highlights".

### 1.1.51 / Contract 1.1.53 — Atom subtree slice 21: simple groups typed (capturing/noncapturing/branch_reset/atomic)

**What changed:** Four group forms now emit typed `{type:"atom", kind:<group_kind>, body:<pattern>}` objects.

```ebnf
capturing_group     = "(" pattern? ")"           -> {type: "atom", kind: "capturing_group", body: $2}
noncapturing_group  = "(?:" pattern? ")"         -> {type: "atom", kind: "noncapturing_group", body: $2}
branch_reset_group  = "(?|" pattern? ")"         -> {type: "atom", kind: "branch_reset_group", body: $2}
atomic_group        = "(?>" pattern? ")"         -> {type: "atom", kind: "atomic_group", body: $2}
                    | "(*atomic:" pattern? ")"   -> {type: "atom", kind: "atomic_group", body: $2}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(abc)` | `["(", <pattern>, ")"]` | `{type:"atom", kind:"capturing_group", body:<pattern>}` |
| `(?:abc)` | `["(?:", <pattern>, ")"]` | `{kind:"noncapturing_group", body:<pattern>}` |
| `(?>abc)` | `["(?>", <pattern>, ")"]` | `{kind:"atomic_group", body:<pattern>}` |
| `(*atomic:abc)` | `["(*atomic:", <pattern>, ")"]` | `{kind:"atomic_group", body:<pattern>}` (same kind as `(?>...)`) |
| `(?|a|b)` | `["(?|", <pattern>, ")"]` | `{kind:"branch_reset_group", body:<pattern>}` |
| `()` / `(?:)` | similar 3-element Sequence | `body: [[], []]` (empty alternation shape from `pattern?` matched-empty) |

**`body` is the raw pattern shape**, not itself typed. Pattern outer typing is a separate slice.

**Atomic group's two syntactic forms both produce `kind:"atomic_group"`.** PCRE2 treats `(?>...)` and `(*atomic:...)` as semantically equivalent; the typed shape doesn't preserve the syntactic origin (consistent with how `property_escape`'s 4 forms all produce `kind:"property"`).

**Atom subtree campaign progress:** 11/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.51 / Contract 1.1.53 Highlights".

### 1.1.50 / Contract 1.1.52 — Atom subtree slice 20: comment_group typed

**What changed:** `comment_group` now emits typed `{type:"atom", kind:"comment", text:<string>}` objects.

```ebnf
comment_group = "(?#" comment_text ")"
                  -> {type: "atom", kind: "comment", text: $2}

# rewritten from `comment_char*` chain:
comment_text  = /([^)]*)/
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?#hello)` | `["(?#", [<comment_char chain>], ")"]` | `{type:"atom", kind:"comment", text:"hello"}` |
| `(?#)` | `["(?#", [], ")"]` (empty Quantified) | `{type:"atom", kind:"comment", text:""}` |
| `(?#multi word comment)` | similar 3-element Sequence | `{text:"multi word comment"}` |
| `(?#with [special] chars)` | similar | `{text:"with [special] chars"}` |

**`text` is always a string.** Empty comments (`(?#)`) emit `text:""` (real empty string), not `[]` from an un-matched optional slot. The `?` after `comment_text` in `comment_group` was dropped because the regex literal accepts the empty match — `comment_text` always succeeds.

**Char-set coverage** of `[^)]*` matches the previous `comment_char*` chain semantics: any char except `)`.

**Atom subtree campaign progress:** 8/25 atom alternatives directly typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.50 / Contract 1.1.52 Highlights".

### 1.1.49 / Contract 1.1.51 — Atom subtree slice 19: python_named_backreference typed

**What changed:** `python_named_backreference` now emits typed `{type:"backreference", kind:"python_named", ref:<name>}` objects.

```ebnf
python_named_backreference = "(?P=" name ")"
                              -> {type: "backreference", kind: "python_named", ref: $2}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `(?P=foo)` | `["(?P=", "foo", ")"]` | `{type:"backreference", kind:"python_named", ref:"foo"}` |
| `(?P=bar_baz)` | similar 3-element Sequence | `{kind:"python_named", ref:"bar_baz"}` |
| `(?P=x)` | similar | `{kind:"python_named", ref:"x"}` |

`name` was already a clean string after slice 11 (named-ref cleanup), so `$2` extracts directly.

**`kind` distinguishes from `\k<...>` even though semantics are equivalent.** PCRE2 treats `(?P=foo)` and `\k<foo>` as the same match operation, but tooling that wants to preserve the syntax origin can dispatch on `kind`. Consumers normalizing across all name-based forms: `kind in {"named", "named_braced", "python_named"}` → "name-based backref"; `ref` is the name string in all three.

**Atom subtree campaign progress:** 7/25 atom alternatives directly typed. **Backreference family typing is now end-to-end across all 5 syntactic forms** (numeric, named, named_braced, subroutine, python_named).

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.49 / Contract 1.1.51 Highlights".

### 1.1.48 / Contract 1.1.50 — Atom subtree slice 18: quoted_literal typed

**What changed:** `quoted_literal` now emits typed `{type:"atom", kind:"quoted_literal", body:<chars>}` objects.

```ebnf
quoted_literal = "\\Q" quoted_literal_char* "\\E"
                  -> {type: "atom", kind: "quoted_literal", body: $2}
```

**Before / after:**

| Source | Before | After |
|---|---|---|
| `\Qhello\E` | `["\\Q", ["h","e","l","l","o"], "\\E"]` | `{type:"atom", kind:"quoted_literal", body:["h","e","l","l","o"]}` |
| `\Q\E` (empty) | `["\\Q", [], "\\E"]` | `{type:"atom", kind:"quoted_literal", body:[]}` |
| `\Qabc def\E` | similar 3-element seq | `{body:["a","b","c"," ","d","e","f"]}` |

`body` is the array of `quoted_literal_char*` matches — one element per char. `quoted_literal_escaped_char` produces 2-char strings (the `\` and the escaped char). Consumers join to recover the literal string; consumers with semantic needs can distinguish escaped vs raw chars from element shapes.

**Atom subtree campaign progress:** 6/25 atom alternatives directly typed; 7/7 escape_unit branches typed.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.48 / Contract 1.1.50 Highlights".

### 1.1.47 / Contract 1.1.49 — Atom subtree slice 17: escape subtree closes (property)

**What changed:** `property_escape` per-branch annotations now emit typed `{type:"escape", kind:"property", name:<string>, negated:<bool>}` objects. `prop_name` and `short_prop_letter` rewritten from chained forms to regex literals so they emit clean string Terminals.

```ebnf
property_escape = "p{" prop_name "}"      -> {type: "escape", kind: "property", name: $2, negated: false}
                | "P{" prop_name "}"      -> {type: "escape", kind: "property", name: $2, negated: true}
                | "p" short_prop_letter   -> {type: "escape", kind: "property", name: $2, negated: false}
                | "P" short_prop_letter   -> {type: "escape", kind: "property", name: $2, negated: true}

# rewritten from `prop_name_chars+` chain:
prop_name = /([A-Za-z0-9 \t\n\r\f\v_:\-=&^]+)/

# rewritten from Or-of-single-chars chain:
short_prop_letter = /([CLMNPSZclmnpsz])/
```

**Before / after:**

| Source | Before (post-slice-16) | After |
|---|---|---|
| `\p{Lu}` | `["\\", ["p{", [["L"], ["u"]], "}"]]` | `{type:"escape", kind:"property", name:"Lu", negated:false}` |
| `\p{Letter}` | similar 3-level chain | `{type:"escape", kind:"property", name:"Letter", negated:false}` |
| `\P{Nd}` | similar w/ leading `"P{"` | `{type:"escape", kind:"property", name:"Nd", negated:true}` |
| `\pL` | `["\\", ["p", "L"]]` | `{type:"escape", kind:"property", name:"L", negated:false}` |
| `\PN` | `["\\", ["P", "N"]]` | `{type:"escape", kind:"property", name:"N", negated:true}` |

**`negated` is a real boolean.** Consumers get `true` / `false` directly from `obj.negated`, no need to inspect leading-token shape (`"p"` vs `"P"`) to infer negation.

**Atom subtree campaign progress:** 5/25 atom alternatives directly typed; **7/7 escape_unit branches typed** (single_byte, simple, control, hex, unicode, octal, property). The escape subtree is now **closed**.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.47 / Contract 1.1.49 Highlights".

### 1.1.46 / Contract 1.1.48 — Atom subtree slice 16: escape subtree continues (octal)

**What changed:** `octal_escape` per-branch annotations now emit typed `{type:"escape", kind:"octal", digits:<octal-string>}` objects. New `octal_escape_short_payload` regex literal for the 1-3-digit bare form. `octal_digits` rewritten from `octal_digit+` chain to regex literal.

```ebnf
octal_escape = "o{" brace_ws? octal_digits brace_ws? "}"  -> {type: "escape", kind: "octal", digits: $3}
             | octal_escape_short_payload                  -> {type: "escape", kind: "octal", digits: $1}
octal_escape_short_payload = /([0-7]{1,3})/

# rewritten from `octal_digit+` chain:
octal_digits = /([0-7]+)/
```

**Before / after:**

| Source | Before (post-slice-15) | After |
|---|---|---|
| `\o{777}` (atom) | `["\\", ["o{", [], <digits chain>, [], "}"]]` | `{type:"escape", kind:"octal", digits:"777"}` |
| `\o{7777}` (atom) | similar 5-level chain | `{type:"escape", kind:"octal", digits:"7777"}` |
| `[\377]` (in class via `class_range_escape_unit`) | `["\\", ["3", ["7"], ["7"]]]` | `{type:"escape", kind:"octal", digits:"377"}` |

**Important caveat — bare `\NNN` at atom-level:** Outside of character classes, `\377` and similar bare-digit forms are parsed as `{type:"backreference", kind:"numeric", index:377}` under the existing PEG-ordered atom alternation (numeric-backref branch shadows bare-octal). Pre-existing behavior; not changed by this slice. PEG cannot express PCRE2's contextual disambiguation ("if NNN ≤ 9 OR there are NNN capture groups, treat as backref; else octal") directly. Disambiguation is left to consumers via post-parse semantic analysis if/when they need atom-level bare-octal support.

**`digits` is a string, not an int.** Consumers parse with `usize::from_str_radix(digits, 8)`. Same constraint as hex/unicode; extending `@transform` is a separate codegen-feature slice.

**Atom subtree campaign progress:** 5/25 atom alternatives directly typed; **6/7 escape_unit branches typed** (single_byte, simple, control, hex, unicode, octal). Only remaining un-typed escape_unit branch: `property_escape`.

**Contract section:** [`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.1.46 / Contract 1.1.48 Highlights".

### 1.1.45 / Contract 1.1.47 — Atom subtree slice 15: escape subtree continues (hex/unicode)

**What changed:** `hex_escape` and `unicode_escape` now emit typed `{type:"escape", kind:<form>, digits:<hex-string>}` objects:

```ebnf
hex_escape = "x" hex_escape_short_payload                    -> {type: "escape", kind: "hex", digits: $2}
           | "x{" brace_ws? hex_digits brace_ws? "}"         -> {type: "escape", kind: "hex", digits: $3}
hex_escape_short_payload = /([0-9A-Fa-f]{1,2})/

unicode_escape = "u{" hex_digits "}" -> {type: "escape", kind: "unicode", digits: $2}

# hex_digits rewritten from `hex_digit+` (multi-element chain) to a regex literal:
hex_digits = /([0-9A-Fa-f]+)/
```

**Consumer impact:** **breaking but correct** — hex and unicode escape atoms now read as field accesses:

| Source | Before | After |
|---|---|---|
| `\xF` | `["\\", ["x", "F", []]]` (3-elem chain) | `{type:"escape", kind:"hex", digits:"F"}` |
| `\xFF` | `["\\", ["x", "F", [["F"]]]]` | `{type:"escape", kind:"hex", digits:"FF"}` |
| `\x{1F}` | `["\\", ["x{", [], <hex_digits chain>, [], "}"]]` | `{type:"escape", kind:"hex", digits:"1F"}` |
| `\x{1F600}` | similar | `{type:"escape", kind:"hex", digits:"1F600"}` |
| `\u{41}` | `["\\", ["u{", <hex_digits chain>, "}"]]` | `{type:"escape", kind:"unicode", digits:"41"}` |
| `\u{1F600}` | similar | `{type:"escape", kind:"unicode", digits:"1F600"}` |

**`digits` is a string, not an int.** Consumers parse with `usize::from_str_radix(digits, 16)`. Int decoding via the rule's `@transform` would require extending the transform machinery (currently `@transform` is hard-coded to `str::parse::<TYPE>().unwrap_or(DEFAULT)`-style; `from_str_radix` needs a different signature). Saved as a separate codegen-feature slice.

**`hex_digits` rewrite** (was `hex_digit+`, now `/([0-9A-Fa-f]+)/`) cascades to clean strings everywhere `hex_digits` is used (the braced hex/unicode forms above). The standalone `hex_digit` rule is retained for any rule that still uses it directly.

**Atom subtree progress:** 5/25 alternatives directly typed; 5/7 escape_unit branches typed (single_byte, simple, control, hex, unicode). Remaining escape_unit branches: octal_escape, property_escape.

**Note on `\u` validator rejection:** PGEN's host-side compile validator currently rejects `\u{...}` escapes ("unsupported regex escape `\u`"). The annotation works correctly when the validator allows the escape through; for inputs the validator rejects, no AST is produced. That validator behavior is pre-existing and out of scope for this slice.

**Contract section:** "Release 1.1.45 / Contract 1.1.47 Highlights".

### 1.1.44 / Contract 1.1.46 — Atom subtree slice 14: escape subtree starts (simple/single_byte/control)

**What changed:** the `escape` rule and 3 of its 7 sub-rules got typed annotations:

```ebnf
escape         = "\\" escape_unit -> $2
single_byte_escape = "C" -> {type: "escape", kind: "single_byte"}
simple_escape  = any_char -> {type: "escape", kind: "shorthand", char: $1}
control_escape = "c" any_char -> {type: "escape", kind: "control", char: $2}
```

`escape` is a transparent wrapper — each sub-rule constructs its own typed `{type:"escape", kind:<form>, ...}` object, and `escape` passes through via `-> $2`.

**Consumer impact:** **breaking but correct** — most common escape forms now emit typed objects:

| Source | Kind | Output |
|---|---|---|
| `\d` `\w` `\s` `\.` `\\` `\n` (any letter/symbol) | `shorthand` | `{type:"escape", kind:"shorthand", char:<c>}` |
| `\C` | `single_byte` | `{type:"escape", kind:"single_byte"}` |
| `\cA` `\cZ` `\cz` | `control` | `{type:"escape", kind:"control", char:<C>}` |
| `\xFF` `\x{1F}` | `hex` (still raw) | un-typed — follow-up slice |
| `\u{1F600}` | `unicode` (still raw) | un-typed — follow-up slice |
| `\377` `\o{777}` | `octal` (still raw) | un-typed — follow-up slice |
| `\p{Lu}` `\PL` | `property` (still raw) | un-typed — follow-up slice |

Pre-fix, every escape produced a deeply-nested chain `["\\", [[[[[<char>]]]]]]` (4-5 levels of un-annotated wrappers around the simple_escape's any_char Terminal). Post-fix, `\d` is a single field read on `obj.kind` + `obj.char`. The chain disappears entirely for the 3 typed branches.

**Limitation — 4 escape_unit branches still raw.** `\xFF`, `\u{...}`, `\377`/`\o{...}`, `\p{...}`/`\PL` still emit their pre-fix raw shapes. Each requires digit-decoding (hex/unicode/octal) or property-name extraction. Follow-up slices will type them one by one.

**Atom subtree progress:** 5/25 alternatives directly typed (anchor, posix_class, posix_word_boundary_alias, backreference, escape outer). 3/7 escape_unit branches typed (single_byte, simple, control).

**Contract section:** "Release 1.1.44 / Contract 1.1.46 Highlights".

### 1.1.43 / Contract 1.1.45 — Atom subtree slice 13: signed_digits typing (backref family fully typed end-to-end)

**What changed:** the `signed_digits` rule now produces a typed `{sign, value}` object:

```ebnf
signed_digits = sign? digits -> {sign: $1, value: $2}
```

`sign` is `"+"`/`"-"` when the optional sign matched, or `[]` (the un-matched `Quantified-?` slot) when no sign was present. `value` is the typed integer from `digits`.

**Consumer impact:** **breaking but correct** — numeric subroutine refs now read `obj.ref.sign` and `obj.ref.value` as named fields:

| Source | Before | After |
|---|---|---|
| `\g<1>` | `ref: [[], 1]` | `ref: {"sign":[], "value":1}` |
| `\g<-2>` | `ref: ["-", 2]` | `ref: {"sign":"-", "value":2}` |
| `\g<+5>` | `ref: ["+", 5]` | `ref: {"sign":"+", "value":5}` |
| `\g{42}` | `ref: [[], 42]` | `ref: {"sign":[], "value":42}` |
| `\g+1` | `ref: ["+", 1]` | `ref: {"sign":"+", "value":1}` |
| `\g-3` | `ref: ["-", 3]` | `ref: {"sign":"-", "value":3}` |

**Backreference family — typed end-to-end:**
- `kind:"numeric"` → `index:<int>`.
- `kind:"named"` / `kind:"named_braced"` → `ref:<string>`.
- `kind:"subroutine"` → `ref:<string>` (named) OR `ref:{sign:..., value:...}` (numeric, typed).

Consumer code is now field reads end-to-end — no more dispatching on `is_string()` vs `is_array()` for the numeric form.

**Atom subtree progress:** 4/25 alternatives directly typed; backreference family fully typed end-to-end.

**Contract section:** "Release 1.1.43 / Contract 1.1.45 Highlights".

### 1.1.42 / Contract 1.1.44 — Atom subtree slice 12: subroutine_ref cleanup (closes backref family)

**What changed:** the `subroutine_ref` rule's 4 branches each got per-branch annotations to drop the angle/quote/brace delimiters and surface the inner `signed_digits_or_name` directly. `braced_subroutine_ref` annotated similarly:

```ebnf
subroutine_ref = braced_subroutine_ref          -> $1
              | "<" signed_digits_or_name ">"   -> $2
              | "'" signed_digits_or_name "'"   -> $2
              | signed_digits                   -> $1
braced_subroutine_ref = "{" brace_ws? signed_digits_or_name brace_ws? "}" -> $3
```

**Consumer impact:** **breaking but correct** — `\g<...>` family backreferences now surface clean inner values:

| Source | Before | After |
|---|---|---|
| `\g<name>` | `ref: ["<", "name", ">"]` (slice 11 cleaned inner) | `ref: "name"` |
| `\g'name'` | `ref: ["'", "name", "'"]` | `ref: "name"` |
| `\g{name}` | `ref: ["{", _, "name", _, "}"]` | `ref: "name"` |
| `\g<1>` | `ref: ["<", [[], 1], ">"]` | `ref: [[], 1]` |
| `\g<-2>` | `ref: ["<", ["-", 2], ">"]` | `ref: ["-", 2]` |
| `\g{42}` | `ref: ["{", _, [[], 42], _, "}"]` | `ref: [[], 42]` |
| `\g+1` | `ref: ["+", 1]` (already raw signed_digits) | `ref: ["+", 1]` (unchanged) |

**Backreference family closed for naming:**
- `kind:"named"` / `kind:"named_braced"` → `ref` is a clean string.
- `kind:"numeric"` → `index` is a typed integer.
- `kind:"subroutine"` → `ref` is either a clean string (named form) or `[<sign?>, <digit-int>]` (numeric form).

**Limitation — `signed_digits` still raw.** `signed_digits = sign? digits` is still un-annotated, so `\g<1>` produces `ref: [[], 1]` — a 2-element array `[<sign?-Quantified>, <typed integer>]`. Consumer dispatches by:
- `obj.ref.is_string()` → it's a name.
- `obj.ref.is_array()` → it's signed_digits; walk `[<sign>, <int>]`.

A future sub-slice will type `signed_digits` to `{sign: <"+"|"-"|null>, value: <int>}` for cleaner consumer ergonomics.

**Atom subtree progress:** 4/25 alternatives directly typed; named-ref + subroutine-ref family fully cleaned (closes backreference deep typing modulo the signed_digits sub-slice).

**Contract section:** "Release 1.1.42 / Contract 1.1.44 Highlights".

### 1.1.41 / Contract 1.1.43 — Atom subtree slice 11: named-ref cleanup (clean name strings)

**What changed:** three grammar changes that surface clean name strings everywhere a name appears:

1. `name` rewritten as a regex literal:
   ```ebnf
   name = /((?:[A-Za-z_]|[^\x00-\x7F])(?:[A-Za-z0-9_]|[^\x00-\x7F])*)/
   ```
   Pre-rewrite the rule was `name_start name_continue*` (multi-element body) which produced a deeply-nested `[first_char, [rest_chars]]` Sequence requiring consumers to concatenate. Post-rewrite the rule emits a Terminal of the matched name string directly.

2. `name_ref` (used by `\k<name>` / `\k'name'` backreferences) annotated to extract just the name:
   ```ebnf
   name_ref = "<" name ">" -> $2
            | "'" name "'" -> $2
   ```

3. `braced_name_ref` (used by `\k{name}`) annotated similarly:
   ```ebnf
   braced_name_ref = "{" brace_ws? name brace_ws? "}" -> $3
   ```

**Consumer impact:** **breaking but correct** — every consumer that walked the raw inner chain to recover a name string now reads the clean string directly:

| Source | Before | After |
|---|---|---|
| `\k<foo>` | `ref: ["<", ["f", ["o", "o"]], ">"]` | `ref: "foo"` |
| `\k{foo}` | `ref: ["{", [], ["f", ["o", "o"]], [], "}"]` | `ref: "foo"` |
| `(?<bar>x)` | atom contains name as `["b", ["a", "r"]]` | atom contains name as `"bar"` |
| `(?P<bar>x)` | similar raw chain | `"bar"` |
| `(?P=bar)` | similar raw chain | `"bar"` |
| `\g<name>` | `ref: ["<", ["n", ["a", "m", "e"]], ">"]` | `ref: ["<", "name", ">"]` (subroutine_ref still un-annotated; inner name now clean) |

**Cascading scope:** this slice affects every grammar rule that references `name` directly or through a wrapper — named groups (5 forms), backreferences (`\k<...>`/`\k{...}`), python named back-refs (`(?P=name)`), subroutine targets (`\g&name`/`\g<name>`/`\g{name}`), conditions (`(?(name)...)` / `(?(R&name)...)`), property escapes' `prop_name` is unrelated and not affected.

**Limitation — `subroutine_ref` still un-annotated.** `\g<name>` etc. still emit the raw `["<", <inner>, ">"]` shape, but the inner now carries a clean name string instead of a character chain. Follow-up slice will type `subroutine_ref` to drop the angle/brace/quote wrappers.

**Atom subtree progress:** 4/25 alternatives directly typed; the named-ref family now emits clean string values everywhere `name` is used.

**Contract section:** "Release 1.1.41 / Contract 1.1.43 Highlights".

### 1.1.40 / Contract 1.1.42 — PGEN-RGX-0077: `[$1**]` flatten-spread peels `Alternative`

**Bug** (RGX bug report PGEN-RGX-0077): every multi-char `\Q...\E quantifier?` source produced one extra wrapping layer at `pattern[0][0]` — `[[<N pieces>]]` (1-element array containing the pieces array) instead of the documented flat `[<N pieces>]`. The piece data was correct; the bug was purely structural wrapping. Adjacent regression to PGEN-RGX-0075 on a different codegen path.

**Root cause:** the `[$1**]` flatten-spread codegen in `rust/src/ast_pipeline/ast_return_transform.rs` did not peel `Alternative` wrapping before inspecting child content for the unwrap decision. The codegen wraps Or-rule and rule-reference branch results in `Alternative(boxed_inner)`; for `concatenation = piece+ -> [$1**]`, each piece node arrives as `Alternative(piece_inner_node)`. Pre-fix, the inner `match node.content` saw `Alternative` and fell into the "push as-is" arm, wrapping the whole inner Sequence-of-pieces (from `piece_quoted_run_quantified -> [$2**, ...]`) as a single element instead of spreading.

**Fix:**
1. Peel `Alternative` recursively in the FlattenSpread codegen before the unwrap decision. Now `Alternative(inner)` → look at `inner.content` to decide how to spread.
2. Add a `ParseContent::Json(Value::Array(_))` arm (preventative — guards against the same family of regressions for any future annotation that builds typed-Json arrays).

**Consumer impact:** every multi-char `\Q...\E quantifier?` source now produces flat pieces at `pattern[0][0]`. Empirical:

| Source | Before | After |
|---|---|---|
| `\Qab*\E{2,}` | `[[3 pieces]]` | `[a, b, *{2,}]` |
| `\Qabc\E?` | `[[3 pieces]]` | `[a, b, c?]` |
| `\Qabcdef\E+` | `[[6 pieces]]` | `[a, b, c, d, e, f+]` |
| `\Qab\E{3}` | `[[2 pieces]]` | `[a, b{3}]` |

Single-char (`\Qa\E{3}`) and empty (`\Q\E{2}`) cases unaffected — they hit the atom-fallback path, not `piece_quoted_run_quantified`.

**Regression-lock test:** `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` in `rust/src/embedding_api.rs` pins the family-table coverage from the bug report (9 multi-char `\Q...\E quantifier?` shapes). Asserts piece count + atom values + quantifier-attached-to-last-piece + no-quantifier-on-inner-pieces.

**Contract section:** "Release 1.1.40 / Contract 1.1.42 Highlights".

### 1.1.39 / Contract 1.1.41 — Atom subtree slice 10: typed `backreference` shape

**What changed:** the `backreference` rule's 4 branches each got per-branch typed annotations:

```ebnf
backreference  = "\\" backreference_digits  -> {type: "backreference", kind: "numeric",      index: $2}
               | "\\k" name_ref             -> {type: "backreference", kind: "named",        ref:   $2}
               | "\\k" braced_name_ref      -> {type: "backreference", kind: "named_braced", ref:   $2}
               | "\\g" subroutine_ref       -> {type: "backreference", kind: "subroutine",   ref:   $2}
```

`backreference_digits` rewritten as a regex literal `/([1-9][0-9]*)/` with `@transform: str::parse::<usize>().unwrap_or(0)` so branch 0's `index` field is a typed integer directly (mirrors how `digits` was typed in slice 1).

**Consumer impact:** **breaking but correct** — consumers walking backreference atoms must update from `["\\", <digits>]` 2-element array dispatch to typed `obj.kind` lookup:

| Source | Before | After |
|---|---|---|
| `\1` | `["\\", ["1"]]` | `{"type":"backreference","kind":"numeric","index":1}` |
| `\23` | `["\\", ["2", ["3"]]]` | `{"type":"backreference","kind":"numeric","index":23}` |
| `\k<foo>` | `["\\k", ["<", <name>, ">"]]` | `{"type":"backreference","kind":"named","ref":[..raw name_ref..]}` |
| `\k{foo}` | `["\\k", ["{", _, <name>, _, "}"]]` | `{"type":"backreference","kind":"named_braced","ref":[..raw braced_name_ref..]}` |
| `\g{2}` | `["\\g", [..]]` | `{"type":"backreference","kind":"subroutine","ref":[..raw subroutine_ref..]}` |

**Limitation:** for branches 1-3 (`named` / `named_braced` / `subroutine`), the `ref` field carries the inner sub-rule's RAW shape — `name_ref`, `braced_name_ref`, and `subroutine_ref` are still un-annotated as of this slice. Consumers walking the name string need to descend the raw chain. A follow-up slice will type those rules so `ref` becomes `{name: <str>}` for named refs and `{kind: <numeric|named|signed_numeric>, value: ...}` for subroutine refs.

**Atom subtree progress:** 4 of 25 alternatives annotated (anchor, posix_class, posix_word_boundary_alias, backreference).

**Contract section:** "Release 1.1.39 / Contract 1.1.41 Highlights".

### 1.1.38 / Contract 1.1.40 — Atom subtree slice 9: typed `posix_word_boundary_alias` (closes anchor family)

**What changed:** the `posix_word_boundary_alias` rule's 2 branches each annotated to emit the same typed anchor shape as the `anchor` rule:

```ebnf
posix_word_boundary_alias = "[[:<:]]" -> {type: "anchor", kind: "posix_word_start"}
                          | "[[:>:]]" -> {type: "anchor", kind: "posix_word_end"}
```

PCRE2's POSIX-style word-boundary aliases (`[[:<:]]` / `[[:>:]]`) are anchors despite the character-class-looking syntax. They now join the typed anchor family — consumers can dispatch uniformly on `obj.type == "anchor"` regardless of whether the source used `\b` (regular) or `[[:<:]]`/`[[:>:]]` (POSIX-style).

**Consumer impact:** **breaking but correct** — consumers walking the `posix_word_boundary_alias` atom must update from `atom.as_str() == "[[:<:]]"` to `atom.get("kind").as_str() == "posix_word_start"` (and similar for end). After this slice, the consumer-side `classify_anchor` recipe in [Examples: Anchors](examples-anchors.md) covers all 11 anchor variants with no fallback paths.

**Anchor family closed:** all 11 anchor variants — 9 from `anchor` (slice 7) + 2 from `posix_word_boundary_alias` (this slice) — emit the same typed `{type:"anchor", kind:<name>}` shape:

| Source | `kind` |
|---|---|
| `^` | `start_of_line` |
| `$` | `end_of_line` |
| `\A` | `start_of_input` |
| `\Z` | `end_of_input_or_before_last_newline` |
| `\z` | `end_of_input` |
| `\b` | `word_boundary` |
| `\B` | `non_word_boundary` |
| `\G` | `match_start` |
| `\K` | `keep_out` |
| `[[:<:]]` | `posix_word_start` |
| `[[:>:]]` | `posix_word_end` |

**Atom subtree progress:** 3 of 25 alternatives annotated (anchor, posix_class, posix_word_boundary_alias). Note: anchor and posix_word_boundary_alias are siblings under `atom`, but together they close the anchor-shaped family (3 grammar atoms emit the same typed shape).

**Contract section:** "Release 1.1.38 / Contract 1.1.40 Highlights".

### 1.1.37 / Contract 1.1.39 — PGEN-RGX-0076: typed `posix_class` shape (slice 8)

**Bug**: RGX bug report PGEN-RGX-0076 — every POSIX class inside a character class collapsed to the literal string `"[:"` in the typed shape. The grammar had a placeholder annotation `posix_class = "[:" posix_negation? posix_name ":]" -> $1` which extracted only the FIRST element (the `"[:"` opener), silently discarding the matched POSIX name and any negation marker.

**Fix**:
```ebnf
posix_class = "[:" posix_negation? posix_name ":]"
-> {type: "posix_class", name: $3, negated: $2}

posix_negation = "^" -> true
```

**Codegen fixes** in the same commit:
- `BooleanLiteral` codegen at the rule-level scalar path (`generate_transform`) was emitting `ParseContent::Terminal(<bool_str>)` — a string Terminal `"true"`/`"false"` — instead of a typed JSON boolean. Surfaced when `posix_negation -> true` produced `"true"` (string) instead of `true` (bool).
- `NumberLiteral` codegen at the same path had the analogous bug. Both now emit `ParseContent::Json(serde_json::Value::Bool/Number(...))` mirroring the value-extraction path.

**Consumer impact:** **breaking but correct**. Every POSIX class inside `[...]` now emits a typed object:

| Source | Before | After |
|---|---|---|
| `[[:alpha:]]` | `class_body[0] = "[:"` | `class_body[0] = {"type":"posix_class","name":"alpha","negated":[]}` |
| `[[:^alpha:]]` | `class_body[0] = "[:"` | `class_body[0] = {"type":"posix_class","name":"alpha","negated":true}` |
| `[[:alpha:][:digit:]]` | `class_body = ["[:", "[:"]` (both truncated identically) | `class_body = [{type:posix_class,name:alpha,negated:[]}, {type:posix_class,name:digit,negated:[]}]` |

Consumers walking the typed shape can drop any source-span fallback they had for POSIX class name recovery — the typed object preserves `name` and `negated` directly.

**`negated` convention:** `true` (matched `^`) or `[]` (un-matched `posix_negation?` slot — map to `false`). Same convention as `quantifier.greediness`. A future coalesce-operator slice will let the rule emit a bare `false` directly.

**Contract section:** "Release 1.1.37 / Contract 1.1.39 Highlights".

### 1.1.36 / Contract 1.1.38 — Atom subtree slice 7: typed `anchor` shape

**What changed:** the `anchor` rule's 9 branches each got `-> {type: "anchor", kind: "<name>"}` annotations. Piece atoms for `^`/`$`/`\A`/`\Z`/`\z`/`\b`/`\B`/`\G`/`\K` now emit typed objects with semantic kind names instead of raw escape strings.

```ebnf
anchor = "^"   -> {type: "anchor", kind: "start_of_line"}
       | "$"   -> {type: "anchor", kind: "end_of_line"}
       | "\\A" -> {type: "anchor", kind: "start_of_input"}
       | "\\Z" -> {type: "anchor", kind: "end_of_input_or_before_last_newline"}
       | "\\z" -> {type: "anchor", kind: "end_of_input"}
       | "\\b" -> {type: "anchor", kind: "word_boundary"}
       | "\\B" -> {type: "anchor", kind: "non_word_boundary"}
       | "\\G" -> {type: "anchor", kind: "match_start"}
       | "\\K" -> {type: "anchor", kind: "keep_out"}
```

**Consumer impact:** **breaking but correct** — consumers dispatching on the raw escape text must switch to the typed `obj.kind` field. The kind names are stable identifiers and won't change if PCRE2 syntax evolves. See [Examples: Anchors and Boundaries](examples-anchors.md) for the full migration recipe.

| Source | Before | After |
|---|---|---|
| `^` | `"atom": "^"` | `"atom": {"type":"anchor","kind":"start_of_line"}` |
| `\b` | `"atom": "\\b"` | `"atom": {"type":"anchor","kind":"word_boundary"}` |
| `\K` | `"atom": "\\K"` | `"atom": {"type":"anchor","kind":"keep_out"}` |

**Note:** the POSIX word-boundary aliases (`[[:<:]]` and `[[:>:]]`, handled by the `posix_word_boundary_alias` rule) still emit raw 7-char terminals. They will join the typed family in a follow-up slice.

**Atom subtree campaign progress:** 1 of 25 alternatives annotated. Next focus areas: `dot`, `literal`, `backreference`, `quoted_literal`, `escape`, `posix_word_boundary_alias`, `char_class`, group family, etc.

**Contract section:** "Release 1.1.36 / Contract 1.1.38 Highlights".

### 1.1.35 / Contract 1.1.37 — Quantifier subtree closure (slice 6/N)

**What changed:** the final two rules in the quantifier subtree got their typed annotations:

- `quant_base` reshaped from per-branch `-> $1` (heterogeneous: string for shorthand, object for counted) to per-branch typed `{min, max}` for every alternative:

  ```ebnf
  quant_base = "*"                -> {min: 0, max: null}
             | "+"                -> {min: 1, max: null}
             | "?"                -> {min: 0, max: 1}
             | counted_quantifier -> $1
  ```

- `quantifier` rule annotated:

  ```ebnf
  quantifier = quant_base quant_suffix?
  -> {type: "quantifier", min: $1.min, max: $1.max, greediness: $2}
  ```

**Consumer impact:** **breaking but correct** — the piece's `quantifier` field is now a fully typed `{type, min, max, greediness}` object instead of a `[<base>, <suffix>]` 2-tuple. Empirical:

| Input | Before | After |
|---|---|---|
| `a*` | `["*", []]` | `{"type":"quantifier","min":0,"max":null,"greediness":[]}` |
| `a+?` | `["+", "lazy"]` | `{"type":"quantifier","min":1,"max":null,"greediness":"lazy"}` |
| `a{2,5}` | `[{"min":2,"max":5}, []]` | `{"type":"quantifier","min":2,"max":5,"greediness":[]}` |

Pieces with NO quantifier still have `"quantifier": []` (empty `quantifier?` slot — unchanged).

`greediness: []` is the un-matched `quant_suffix?` slot — interpret as PCRE2's greedy default. Consumers map `[]` → `"greedy"`. This will be removed when the annotation language gains a coalesce operator and `quantifier`'s annotation can emit the literal string `"greedy"` directly.

**Quantifier-subtree campaign closed:** all six rules (`digits`, `quant_suffix`, `counted_quantifier_body`, `counted_quantifier`, `quant_base`, `quantifier`) are now annotated. Consumer-side `extract_quantifier` walker collapses to a six-line typed-field read.

**Contract section:** "Release 1.1.35 / Contract 1.1.37 Highlights".

### 1.1.34 / Contract 1.1.36 — PGEN-RGX-0075 typed-shape correctness for multi-piece concatenation

**What changed:** The `$N` PositionalRef codegen no longer peels `elements[0]` from a `Quantified` base when the rule body has a single capture position. With this fix, `concatenation = piece+ -> [$1**]` correctly resolves `$1` to the whole `Quantified` (every piece), so `**` flattens all of them into the array.

Compensating grammar change: `regex = pattern -> ...` (was `regex = pattern? -> ...`). The inner `alternative = concatenation?` already handles the empty-input case, so the outer `?` was redundant and only existed to prop up the buggy auto-peel behaviour.

**Consumer impact:** **breaking but in the right direction** — anyone walking `regex.pattern[0][0]` for a multi-piece concatenation now sees every piece. Pre-fix, `"abc"` produced `pattern[0][0] == [piece_a]` (1 piece, buggy); post-fix, `pattern[0][0] == [piece_a, piece_b, piece_c]` (3 pieces, correct). The top-level pattern shape `[<head_alt>, <tail>]` is unchanged. RGX caught this when `Regex::compile("abc")` matched only `"a"` instead of `"abc"`.

**Bug history:** the `\Q...\E` family table covered `\Qab*\E{2,}` (3 pieces — passed incidentally because `piece_quoted_run_quantified`'s annotation pre-built a multi-element Sequence that `**` happened to flatten correctly), but no test asserted plain `"abc"` → 3 pieces in the typed output. PGEN-RGX-0074's empirical evidence focused on `\Q...\E` cases, which masked the underlying `$1`-extraction bug. Fixed in this release with a regression-lock test (`regex_parser_pgen_rgx_0075_multi_piece_concatenation_surfaces_all_pieces`) pinning the empirical shape for `"a"`, `"ab"`, `"abc"`, `"hello"`.

**Contract section:** "Release 1.1.34 / Contract 1.1.36 Highlights".

### Post-1.1.33 main — Task #38 fix: parens-grouped-Or trailing-annotation broadcast

**What changed:** the codegen now correctly applies a trailing return annotation on a parens-grouped Or to **every** alternative inside the group, not just the first. Affects both `extract_rule_annotations` (rust/src/ast_pipeline/mod.rs) and the cross-checker `extract_declared_annotations_from_json` (rust/src/ast_shape_contract.rs).

Pre-fix behaviour: `RULE = (A | B | C) -> ann` applied `ann` to branch 0 only; branches 1, 2 silently fell through to raw passthrough. Documented in `parse_string_literal` of the return_annotation grammar — single-quoted strings produced raw `Sequence` instead of the typed `{type:"string", value:...}` that double-quoted strings produced.

Post-fix behaviour: when a return annotation immediately follows a `group_close`, the annotation broadcasts to every branch that was inside the just-closed group. Per-branch annotations on un-grouped Or rules (`A | B -> ann`, where `-> ann` binds to B only per PEG precedence) still work as before.

**Consumer impact for return_annotation grammar:** single-quoted strings now produce `Json({"type":"string", "value":"..."})` — same as double-quoted. Anyone relying on the buggy raw-Sequence shape needs to update walking code.

**Consumer impact for regex grammar:** none directly from the bugfix. But `quant_base` was refactored to the now-supported factored form `( "*" | "+" | "?" | counted_quantifier ) -> $1` (was per-branch `-> $1` four times) — same JSON output.

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — quant_base annotated (slice 5/N)

**What changed:** `quant_base = "*" | "+" | "?" | counted_quantifier` got per-branch `-> $1` annotations on every alternative. After task #38 fix landed (subsequent commit), the rule was refactored to the factored form `quant_base = ( "*" | "+" | "?" | counted_quantifier ) -> $1` — semantically identical, more elegant.

**Consumer impact:** **none** — JSON output is byte-identical to pre-slice-5. Empirical: `a*` still emits `quantifier: ["*", []]`; `a{2,5}` still emits `quantifier: [{"min":2,"max":5}, []]`. The change is to the rule's emission status (from "raw envelope via codegen default" to "annotated, Tier-2 stable").

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — counted_quantifier typed (slice 4/N)

**What changed:** `counted_quantifier` rule got `-> $3` annotation, lifting `counted_quantifier_body`'s typed `{min, max}` straight through and dropping the surrounding `{`/`}`/whitespace tokens. The brace tokens carry no semantic information beyond "this is a counted quantifier" — context the surrounding `quant_base` already conveys.

**Consumer impact:** the `quant_base` position (visible inside `quantifier`'s `[<base>, <suffix>]` shape) now carries either a bare string `"*"`/`"+"`/`"?"` OR a typed `{min, max}` object directly. No more digging through a 5-element Sequence wrapper to reach the body's typed shape. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — counted_quantifier_body typed + null literal (slice 3/N)

**What changed:** restructured `counted_quantifier_body` from 2 branches (with 4 logical cases compressed inside an optional sub-group) into 4 explicit branches each with its own per-branch `-> {min, max}` annotation. Added the `null` literal to the return-annotation language so the unbounded `{n,}` form can encode `max:null` directly.

**Consumer impact:** the body now emits a typed `{min, max}` object regardless of which `{n}`/`{n,}`/`{n,m}`/`{,m}` source form matched. `min` is always a typed integer; `max` is a typed integer OR `null` (only `null` for the unbounded form). See [Quantifier Subtree](rules-quantifier.md), [Quantifiers](examples-quantifiers.md).

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### 1.1.33 / Contract 1.1.35 — quant_suffix typed (slice 2/N)

**What changed:** `quant_suffix` rule's two branches each got a return annotation: `"?"` → `"lazy"`, `"+"` → `"possessive"`. The `quant_suffix?` slot inside `quantifier` now carries either `[]` (greedy, no suffix matched) or `["lazy"]` / `["possessive"]` (1-element Quantified-?).

**Consumer impact:** consumers reading the quantifier shape can dispatch on the string instead of inferring from which branch matched. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** "Release 1.1.33 / Contract 1.1.35 Highlights".

### 1.1.32 / Contract 1.1.34 — digits typed (slice 1/N)

**What changed:** `digits` rule got `@transform: str::parse::<usize>().unwrap_or(0)` annotation. The rule now emits a typed integer (`Json(Value::Number(usize))`) instead of a `Terminal` of digit characters.

**Consumer impact:** consumers reading counted-quantifier bounds get the integer directly. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** "Release 1.1.32 / Contract 1.1.34 Highlights".

### 1.1.31 / Contract 1.1.33 — PGEN-RGX-0074 `\Q...\E` correctness

**What changed:** introduced the `piece_quoted_run_quantified` rule + the `**` flatten-spread operator. Restructured `concatenation` to `piece+ -> [$1**]`. The `\Q...\E quantifier` shape now emits per-char piece array with the trailing piece carrying the quantifier — matching PCRE2's "quantifier binds to last char of \Q...\E" semantics.

**Consumer impact:** **breaking** — any consumer that walked the pre-1.1.31 whole-block-quantified shape will see a different AST. The new shape is what PCRE2 says it should be. Drop any pre-existing `\Q...\E`-quantifier-attachment workaround. See [\Q...\E Quoted Literals](examples-quoted-literal.md).

**Contract section:** "Release 1.1.31 / Contract 1.1.33 Highlights".

### 1.1.30 / Contract 1.1.32 — PGEN-RGX-0073 perf closure

**What changed:** primary focus was performance — closing all 8 patterns under p50 < 50µs. Schema-affecting changes were minor: implicit `-> $1` default tightened to exclude `Quantified` bodies (so single-Quantified rules without an explicit annotation no longer collapse to the inner item). Several rule rewrites of slow Or-of-chars rules using `/.../` regex literals (perf-only, no shape change).

**Consumer impact:** if a consumer relied on the implicit-collapse behavior for a Quantified body, they need to add an explicit `-> $1` annotation to that rule (or update their walker to handle the un-collapsed shape). Vanishingly rare in practice — the regex grammar didn't have any rules in this position. See contract for full details.

**Contract section:** "Release 1.1.30 / Contract 1.1.32 Highlights".

## Earlier releases (pre-1.1.30)

Pre-1.1.30 the regex parser used the recursive-envelope shape exclusively — no `Json` carrier, every rule produced `Sequence` / `Alternative` / `Quantified` / `Terminal`. Consumers who built against pre-1.1.30 should read [Migration from the Recursive Envelope](migration-from-recursive-envelope.md) before reading the rest of this book.

For shape-affecting changes within the pre-1.1.30 envelope era, see the contract document — its release sections go back to the parser's earliest releases.

## Future releases

Upcoming slice campaign work (task #40 — "Annotate regex.ebnf for full AST usability"):

| Slice | Target rule | Expected AST change |
|---|---|---|
| 6 | `quantifier` | Combine quant_base + quant_suffix? into typed `{type:"quantifier", min, max, greediness}` |
| 7+ | atom subtree, char class, group family, escape subtree, ... | One rule per slice |

Each slice will:

- Bump the contract version.
- Add a row to this index.
- Get a CHANGES.md entry.
- Get a corresponding chapter update where applicable.

The eventual schema 1.0.0 milestone will land when every regex.ebnf rule is either annotated or has a deliberate "remain raw envelope" decision documented.

## Looking up a specific input's behavior

If you need to know "what does the parser output for input X across release Y?," the workflow is:

1. Check `git log --oneline grammars/regex.ebnf` to see when the rule for X was last touched.
2. Check the contract section for that release.
3. Check the ledger if there was a bug fix involved.
4. As last resort, check out the relevant tag and run `parseability_probe --parse-dump-ast-pretty regex 'X'` against it.

For consumer-side regression detection, the recommended pattern is the snapshot test described in [Schema Versioning](schema-versioning.md).
