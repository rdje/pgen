# Group Family

PCRE2 has a rich set of group constructs — capturing, non-capturing, named, atomic, branch-reset, lookarounds, conditionals, scan-substring groups, script-run groups, and subroutine calls. None are currently annotated in `regex.ebnf`. All emit raw envelope shapes.

## `group`

```ebnf
group = capturing_group | noncapturing_group | named_group | python_named_group
```

4-way Or. Each branch's content varies.

## `capturing_group`

```ebnf
capturing_group = "(" pattern? ")"
```

Plain capturing group. 3-element Sequence: `["(", <pattern?>, ")"]`.

For `(abc)`:

```json
[
  "(",
  [<pattern array for abc>],
  ")"
]
```

`pattern?` is wrapped in a Quantified-`?`, so the actual pattern content is at `[1][0]` when matched.

## `noncapturing_group`

```ebnf
noncapturing_group = "(?:" pattern? ")"
```

3-element Sequence starting with `"(?:"`.

For `(?:abc)`:

```json
[
  "(?:",
  [<pattern array for abc>],
  ")"
]
```

## `named_group`

```ebnf
named_group = "(?<" name ">" pattern? ")"
            | "(?'" name "'" pattern? ")"
```

2 branches — angle-bracket form and apostrophe form. 5-element Sequence.

For `(?<name>abc)`:

```json
[
  "(?<",
  <name shape>,
  ">",
  [<pattern array for abc>],
  ")"
]
```

The `name` shape is the per-rule output (see `name` below).

## `python_named_group`

```ebnf
python_named_group = "(?P<" name ">" pattern? ")"
```

Python-style named group. 5-element Sequence starting `"(?P<"`.

## `atomic_group`

```ebnf
atomic_group = "(?>" pattern? ")" | "(*atomic:" pattern? ")"
```

2-way Or. Both forms produce 3-element Sequences.

For `(?>foo)`: `["(?>", [<pattern>], ")"]`.

For `(*atomic:foo)`: `["(*atomic:", [<pattern>], ")"]`.

## `branch_reset_group`

```ebnf
branch_reset_group = "(?|" pattern? ")"
```

3-element Sequence: `["(?|", <pattern?>, ")"]`.

## `lookaround`

```ebnf
lookaround = lookahead_pos | lookahead_neg | lookbehind_pos | lookbehind_neg
           | non_atomic_lookahead_pos | non_atomic_lookbehind_pos | alpha_lookaround
```

7-way Or. Each branch is a 3-element Sequence with a different opening prefix.

| Branch | Form | Shape |
|---|---|---|
| 0 (`lookahead_pos`) | `(?=...)` | `["(?=", <pattern>, ")"]` |
| 1 (`lookahead_neg`) | `(?!...)` | `["(?!", <pattern>, ")"]` |
| 2 (`lookbehind_pos`) | `(?<=...)` | `["(?<=", <pattern>, ")"]` |
| 3 (`lookbehind_neg`) | `(?<!...)` | `["(?<!", <pattern>, ")"]` |
| 4 (`non_atomic_lookahead_pos`) | `(?*...)` | `["(?*", <pattern>, ")"]` |
| 5 (`non_atomic_lookbehind_pos`) | `(?<*...)` | `["(?<*", <pattern>, ")"]` |
| 6 (`alpha_lookaround`) | `(*pla:...)`, `(*nla:...)`, `(*plb:...)`, `(*nlb:...)`, `(*napla:...)`, `(*naplb:...)` and full names | `["(*", <name>, ":", <pattern?>, ")"]` |

`alpha_lookaround_name` is itself a 2-way Or between atomic and non-atomic alpha forms — see the rules in `regex.ebnf`.

## `subroutine_call`

```ebnf
subroutine_call = "(?" returned_capture_subroutine ")"
                | "(?" subroutine_target ")"
```

2-way Or. The matched form's body is the returned-capture or subroutine target wrapped in `("(?", <body>, ")")`.

`subroutine_target` is itself a 4-way Or:
- `&name` — named subroutine.
- `P>name` — Python-style.
- `R` — recursion.
- `<signed_digits>` — numeric reference.

## `scan_substring_group`

```ebnf
scan_substring_group = "(*" scan_substring_name ":" returned_capture_group_list pattern? ")"
```

5-element Sequence starting `(*scs:` or `(*scan_substring:`.

## `script_run_group`

```ebnf
script_run_group = "(*" script_run_name ":" pattern? ")"
```

5-element Sequence starting `(*sr:`, `(*script_run:`, `(*asr:`, or `(*atomic_script_run:`.

## `conditional`

```ebnf
conditional = "(?(" condition ")" yes_branch ("|" no_branch)? ")"
```

The PCRE2 conditional group. Up to 6 elements: `["(?(", <condition>, ")", <yes_branch>, <no_branch?>, ")"]`.

`condition` is a 9-way Or covering DEFINE, VERSION, callout-prefixed assertion, regular assertion, name reference, recursion condition, name, signed digits, plain digits.

## `python_named_backreference`

```ebnf
python_named_backreference = "(?P=" name ")"
```

3-element Sequence `["(?P=", <name>, ")"]`.

## Auxiliary rules

### `name`

```ebnf
name = name_start name_continue*
```

A 2-element Sequence `[<first-char>, <Quantified of remaining chars>]`. Each char is a Terminal.

Consumer extraction: concatenate all chars to form the name string.

### `name_ref`

```ebnf
name_ref = "<" name ">" | "'" name "'"
```

2-way Or. 3-element Sequence with delimiters.

### `braced_name_ref`

```ebnf
braced_name_ref = "{" brace_ws? name brace_ws? "}"
```

5-element Sequence `["{", <ws?>, <name>, <ws?>, "}"]`.

### `subroutine_ref`

```ebnf
subroutine_ref = braced_subroutine_ref
              | "<" signed_digits_or_name ">"
              | "'" signed_digits_or_name "'"
              | signed_digits
```

4-way Or. The matched form's content varies.

### `signed_digits`

```ebnf
signed_digits = sign? digits
```

2-element Sequence `[<sign?>, <digits>]`. Recall that `digits` is annotated to emit a typed integer; the `sign` is `+`, `-`, or empty.

### `signed_digits_or_name`, `name_start`, `name_continue`, `brace_ws`, `sign`

Inner sub-rules. Each emits its raw Terminal/Sequence shape per the grammar form.

## Walking a `(?P<foo>bar)` example

For input `(?P<foo>bar)`:

```json
"atom": [
  "(?P<",
  [<name shape — chars: ["f", "o", "o"]>],
  ">",
  [<pattern array for "bar">],
  ")"
]
```

A consumer extracting the name:

```rust
fn extract_name(name_value: &Value) -> String {
    // name = name_start name_continue*
    // → 2-element array: [<first-char>, <Quantified of remaining>]
    let arr = name_value.as_array().unwrap();
    let first = arr[0].as_str().unwrap_or("");
    let rest_arr = arr[1].as_array().map(|v| v.as_slice()).unwrap_or(&[]);
    let mut s = String::new();
    s.push_str(first);
    for c in rest_arr {
        if let Some(ch) = c.as_str() {
            s.push_str(ch);
        }
    }
    s
}
```

## Future direction

The group-family rules will eventually be annotated as part of task #40's atom-subtree slice. Expected target shapes:

- `capturing_group` → `{type: "group", kind: "capturing", body: <pattern>}`.
- `named_group` → `{type: "group", kind: "named", name: <str>, body: <pattern>}`.
- `lookaround` → `{type: "lookaround", direction: "ahead"|"behind", polarity: "positive"|"negative", body: <pattern>}`.
- `conditional` → `{type: "conditional", condition: <typed-cond>, yes: <pattern>, no: <pattern?>}`.
- etc.

Until those annotations land, consumers walk the current Sequence shapes per the per-rule shape table above.
