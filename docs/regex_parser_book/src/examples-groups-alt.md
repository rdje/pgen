# Examples: Groups and Alternations

Concrete probe outputs for grouping and alternation constructs. As of slice 22 (post-1.1.52), **all 6 group sub-rules are typed** — capturing, noncapturing, named (angle + quote), python_named (under `group`), branch_reset, atomic (standalone). Group typing is end-to-end. Recursive-pattern groups (lookarounds, conditionals, scan_substring, script_run) still emit raw envelope shapes pending follow-up slices.

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
| `(?=foo)` | `["(?=", <pattern>, ")"]` |
| `(?!foo)` | `["(?!", <pattern>, ")"]` |
| `(?<=foo)` | `["(?<=", <pattern>, ")"]` |
| `(?<!foo)` | `["(?<!", <pattern>, ")"]` |

```json
"atom": [
  "(?=",
  [<pattern for "foo">],
  ")"
]
```

All four forms produce 3-element Sequences distinguished by the opening-prefix literal at index 0.

## Non-atomic lookahead — `(?*foo)`

```json
"atom": [
  "(?*",
  [<pattern>],
  ")"
]
```

## Non-atomic lookbehind — `(?<*foo)`

```json
"atom": [
  "(?<*",
  [<pattern>],
  ")"
]
```

## Alpha-form lookarounds — `(*pla:foo)`, `(*nla:foo)`, `(*plb:foo)`, etc.

These match `alpha_lookaround` (one of the `lookaround` Or branches):

```json
"atom": [
  "(*",
  "pla",                      // alpha_lookaround_name
  ":",
  [<pattern>],
  ")"
]
```

The `alpha_lookaround_name` rule's content is one of `"pla"`, `"positive_lookahead"`, `"nla"`, `"negative_lookahead"`, `"plb"`, `"positive_lookbehind"`, `"nlb"`, `"negative_lookbehind"`, `"napla"`, `"non_atomic_positive_lookahead"`, `"naplb"`, `"non_atomic_positive_lookbehind"`. Consumers map by name.

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

## Code block — `(?{lua: print(1)})`

```json
"atom": [
  "(?{",
  "lua",
  ":",
  [],                       // optional ws
  [<balanced-brace code content>],
  "})"
]
```

The code content is a Quantified-`*` of `code_element`s. Each element is one of `code_string_double`, `code_string_single`, `code_balanced_braces`, `code_escaped_char`, `code_regular_char` — see grammar `regex.ebnf` lines covering `code_element`.

For consumer purposes, the easiest extraction is via the source span of the code_block atom.
