# Examples: Groups and Alternations

Concrete probe outputs for grouping and alternation constructs. As of slice 23 (post-1.1.53), **all 6 group sub-rules** AND **all 7 lookaround sub-rules** are typed. Group typing is end-to-end. Lookaround family typing is end-to-end. Conditionals, scan_substring, and script_run still emit raw envelope shapes pending follow-up slices.

## Capturing group — `(abc)`

```json
{
  "atom": {"type": "atom", "kind": "capturing_group", "body": <pattern>},
  "quantifier": [],
  "type": "piece"
}
```

`body` is the inner `pattern` shape. For empty `()` it's the empty alternation `[[], []]`. Consumer reads `obj.kind` for dispatch and `obj.body` for the inner pattern (recurse with the same regex-walking logic used for the top-level pattern).

## Non-capturing group — `(?:abc)`

```json
{
  "atom": {"type": "atom", "kind": "noncapturing_group", "body": <pattern>},
  ...
}
```

Identical shape to capturing group except `kind`. Consumer dispatches on `obj.kind`.

## Named group (angle form) — `(?<name>abc)`

```json
{
  "atom": {"type": "atom", "kind": "named_group", "name": "name", "body": <pattern>},
  ...
}
```

`name` is a clean string (typed by slice 11). `body` is the inner pattern shape. For empty body `(?<n>)`, `body: [[], []]` (empty alternation).

## Named group (apostrophe form) — `(?'name'abc)`

```json
{
  "atom": {"type": "atom", "kind": "named_group", "name": "name", "body": <pattern>},
  ...
}
```

Same `kind:"named_group"` as the angle form — PCRE2 treats them as semantically equivalent so the typed shape doesn't preserve the syntactic origin.

## Python-style named — `(?P<name>abc)`

```json
{
  "atom": {"type": "atom", "kind": "python_named_group", "name": "name", "body": <pattern>},
  ...
}
```

Distinct `kind:"python_named_group"`, paralleling `python_named_backreference` (slice 19). PCRE2 treats `(?P<n>...)` as functionally equivalent to `(?<n>...)`, but tooling that displays the source pattern wants to preserve the syntactic origin. Consumers normalizing all name-based group forms: `kind in {"named_group", "python_named_group"}` → name-based group; `name` is the name in both.

## Atomic group — `(?>abc)`

```json
{
  "atom": {"type": "atom", "kind": "atomic_group", "body": <pattern>},
  ...
}
```

## Alpha-form atomic — `(*atomic:abc)`

```json
{
  "atom": {"type": "atom", "kind": "atomic_group", "body": <pattern>},
  ...
}
```

Both syntactic forms produce `kind:"atomic_group"` — PCRE2 treats them as semantically equivalent so the typed shape doesn't preserve the syntactic origin.

## Branch reset — `(?|a|b|c)`

```json
{
  "atom": {"type": "atom", "kind": "branch_reset_group", "body": <pattern>},
  ...
}
```

The inner `pattern` carries the alternation itself (raw `[[<head_alt>], [<tail_alts>]]` shape — pattern outer typing is a separate slice).

## Alternation (top-level) — `a|b`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" }
        ]],
        [
          [
            "|",
            [[
              { "atom": "b", "quantifier": [], "type": "piece" }
            ]]
          ]
        ]
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 3 }
}
```

`pattern[0]` is the first alternative. `pattern[1]` is the `("|" alternative)*` tail — a `Quantified-*` of `[",", <alternative>]` pairs (with `","` here being the literal `"|"` separator, and the alternative content at index 1 of each pair).

## Alternation (3-way) — `a|b|c`

```json
"pattern": [
  [[
    { "atom": "a", "quantifier": [], "type": "piece" }
  ]],
  [
    [
      "|",
      [[
        { "atom": "b", "quantifier": [], "type": "piece" }
      ]]
    ],
    [
      "|",
      [[
        { "atom": "c", "quantifier": [], "type": "piece" }
      ]]
    ]
  ]
]
```

Two `[",", <alternative>]` pairs in the tail. Each pair's index 1 is the alternative's pattern.

A consumer extracting all alternatives:

```rust
fn extract_alternatives(regex: &Value) -> Vec<&Value> {
    let pattern = regex.get("pattern").and_then(|p| p.as_array()).unwrap_or(&[]);
    let mut alts = vec![];
    if let Some(first) = pattern.first() { alts.push(first); }
    if let Some(rest) = pattern.get(1).and_then(|v| v.as_array()) {
        for pair in rest {
            if let Some(p) = pair.as_array() {
                if let Some(alt) = p.get(1) { alts.push(alt); }
            }
        }
    }
    alts
}
```

## Lookarounds — `(?=foo)`, `(?!foo)`, `(?<=foo)`, `(?<!foo)`

| Form | Atom shape |
|---|---|
| `(?=foo)` | `{type:"atom", kind:"lookahead",  positive:true,  body:<pattern>}` |
| `(?!foo)` | `{type:"atom", kind:"lookahead",  positive:false, body:<pattern>}` |
| `(?<=foo)` | `{type:"atom", kind:"lookbehind", positive:true,  body:<pattern>}` |
| `(?<!foo)` | `{type:"atom", kind:"lookbehind", positive:false, body:<pattern>}` |

```json
"atom": {"type": "atom", "kind": "lookahead", "positive": true, "body": <pattern>}
```

`lookahead`/`lookbehind` each collapse 2 syntactic forms (`_pos`/`_neg`) to one `kind` with a `positive` boolean — same convention as `property_escape`'s `negated` field.

## Non-atomic lookahead — `(?*foo)`

```json
"atom": {"type": "atom", "kind": "non_atomic_lookahead", "positive": true, "body": <pattern>}
```

## Non-atomic lookbehind — `(?<*foo)`

```json
"atom": {"type": "atom", "kind": "non_atomic_lookbehind", "positive": true, "body": <pattern>}
```

PCRE2 only supports positive variants of non-atomic lookarounds. The `positive:true` field is included for consumer-code uniformity even though it's always true.

## Alpha-form lookarounds — `(*pla:foo)`, `(*nla:foo)`, `(*plb:foo)`, etc.

```json
"atom": {"type": "atom", "kind": "alpha_lookaround", "name": "pla", "body": <pattern>}
```

`name` carries the alpha_lookaround_name (one of `"pla"`, `"positive_lookahead"`, `"nla"`, `"negative_lookahead"`, `"plb"`, `"positive_lookbehind"`, `"nlb"`, `"negative_lookbehind"`, `"napla"`, `"non_atomic_positive_lookahead"`, `"naplb"`, `"non_atomic_positive_lookbehind"`). Consumers map by name to dispatch on the semantic equivalent.

## Conditional — `(?(cond)yes|no)`

```json
"atom": [
  "(?(",
  <condition shape>,
  ")",
  <yes_branch shape>,
  [
    "|",
    <no_branch shape>
  ],
  ")"
]
```

`condition` has 9 sub-forms — see [Anchors, Backreferences, and Misc](rules-misc.md). For `(?(1)yes|no)`:

- `condition` is just `1` (the digits form, typed integer).
- `yes_branch` is the inner pattern for `yes`.
- `no_branch` (in the optional `("|" no_branch)?` slot) is the pattern for `no`.

## Subroutine call — `(?P>name)`, `(?R)`, `(?1)`

```json
"atom": [
  "(?",
  <subroutine_target shape>,
  ")"
]
```

The subroutine_target distinguishes: `&name`, `P>name`, `R`, or signed digits. See [Group Family](rules-groups.md).

## Code block — `(?{lua: print(1)})` (typed)

```json
"atom": {
  "type": "atom",
  "kind": "code_block",
  "lang": "lua",
  "content": [/* per-char strings or matched code_element shapes */]
}
```

The code content is a Quantified-`*` of `code_element`s. Each element is one of `code_string_double`, `code_string_single`, `code_balanced_braces`, `code_escaped_char`, `code_regular_char` — see grammar `regex.ebnf` lines covering `code_element`.

### Two carrier forms

| Pattern | `lang` | `content` |
|---|---|---|
| `(?{ check_env })` (Perl-style) | `null` | `[" ","c","h","e","c","k","_","e","n","v"," "]` |
| `(?{native:check_env})` | `"native"` | `["c","h","e","c","k","_","e","n","v"]` (since PGEN-RGX-0082 fix) |
| `(?{lua: print(1)})` | `"lua"` | `[" ","p","r","i","n","t","(","1",")"]` |

**PGEN-RGX-0082 note:** in releases 1.1.74 and earlier, the `lang:` branch (`code_block_lang`) had an off-by-one positional reference (`content: $4` referenced the optional `ws?` slot, not the actual `code_content`). For `(?{native:NAME})` patterns: `content` was always `[]`, silently dropping the callback name. Fixed in 1.1.75 (`$4` → `$5`); both `(?{ NAME })` and `(?{lang:NAME})` now produce parallel typed shapes with content preserved.

### Consumer extraction

```rust
fn extract_code_block(atom: &Value) -> Option<(Option<String>, String)> {
    let obj = atom.as_object()?;
    if obj.get("kind")?.as_str()? != "code_block" {
        return None;
    }
    let lang = obj.get("lang")?.as_str().map(String::from);
    // content is per-char array; concat to recover the code body string.
    let content_arr = obj.get("content")?.as_array()?;
    let body: String = content_arr.iter()
        .filter_map(|v| v.as_str())
        .collect();
    Some((lang, body))
}
```

For round-trip / source-span use cases, the easiest extraction is via the source span of the code_block atom (once `_meta.span` lands per the approved `_meta` carrier design).
