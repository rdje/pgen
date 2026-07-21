# Group Family

PCRE2 has a rich set of group constructs — capturing, non-capturing, named, atomic, branch-reset, lookarounds, conditionals, scan-substring groups, script-run groups, and subroutine calls. **The whole family is annotated** (the atom-subtree typed-shape campaign): every group construct emits a typed `{type: "atom", kind: ..., ...}` object with the inner pattern (where there is one) in a `body` field. Names and numeric references are folded into clean string / typed-object fields — consumers never walk raw name/`signed_digits` sequences.

## Parenthesis nesting limit (since release 1.1.77, PGEN-RGX-0085)

Every `(` form drives one recursive-descent frame-chain
(`parse_group → parse_capturing_group → parse_pattern → …`), so the
parser's recursion depth tracks the pattern's `(`-nesting 1:1. To
bound that recursion deterministically, the regex embedding API
enforces a **parenthesis-nesting ceiling** *before* the parser is
invoked:

- **Limit: 250** — the exact PCRE2 `PCRE2_CONFIG_PARENSLIMIT` and
  Rust `regex` crate `nest_limit` default. This is far beyond any
  realistic pattern (real-world regexes rarely nest more than a
  handful of parentheses).
- A pattern whose `(`-group nesting **exceeds 250** is rejected with
  a clean `ParseDiagnostic` (`code: "E_PARSE_FAILURE"`) whose
  `location` (`byte_offset` / `line` / `column`) points at the `(`
  that crossed the limit — exactly PCRE2's "parentheses are too
  deeply nested" behaviour. The recursive-descent parser is **never
  invoked** on over-nested input, so it cannot overflow the stack or
  abort the host process.
- Escaped parentheses (`\(`, `\)`) and parentheses inside a `[...]`
  character class are literals and do **not** count toward the
  nesting depth (character classes cannot nest).
- Nesting **≤ 250 is unaffected**: the typed AST / JSON dump is
  byte-identical to prior releases. This is purely a robustness
  guard — it converts a former host-process abort into a recoverable
  error; it changes no successful parse.

Examples:

```text
# Within the limit — parses normally (depth 64):
((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((a)*)*)*…
  →  ParseStatus::Success  (AST/dump identical to any prior release)

# Over the limit (251+ nested groups):
(((… 251 levels …(a)*…)*)*
  →  ParseStatus::Failure
     diagnostic.code     = "E_PARSE_FAILURE"
     diagnostic.message  = "regex parenthesis nesting exceeds the
                            maximum supported depth of 250 …"
     diagnostic.location = { byte_offset: <offset of the 251st `(`>,
                             line, column }
```

Prior to 1.1.77 the over-limit case overflowed the thread stack and
aborted the host process (`SIGABRT`) — see bug ledger `REGEX-0084`
(downstream `PGEN-RGX-0085`).

## The `body` field — the inner pattern carrier

Every group construct that embeds a pattern carries it in `body`, using the standard
`pattern` carrier `[<first-alternative>, <("|" alternative)* tail>]` — walk it exactly
like the top-level pattern (see [Walking the AST](walking-the-ast.md) and
[Examples: Groups and Alternations](examples-groups-alt.md)). An empty body
(`()`, `(?:)`) is the empty alternation `[[], []]`.

## `group`

```ebnf
group = capturing_group | noncapturing_group | named_group | python_named_group
```

4-way Or; each branch emits its own typed carrier, so the `group` rule itself adds
nothing — consumers dispatch on the atom's `kind`.

## `capturing_group`

```ebnf
capturing_group = "(" pattern? ")"
```

For `(abc)` (exact probe output, `body` abbreviated):

```json
{ "type": "atom", "kind": "capturing_group",
  "body": [[[
    { "atom": "a", "quantifier": [], "type": "piece" },
    { "atom": "b", "quantifier": [], "type": "piece" },
    { "atom": "c", "quantifier": [], "type": "piece" }
  ]], []] }
```

## `noncapturing_group`

```ebnf
noncapturing_group = "(?:" pattern? ")"
```

Same carrier with `kind: "noncapturing_group"`:

```json
{ "type": "atom", "kind": "noncapturing_group", "body": <pattern> }
```

## `named_group`

```ebnf
named_group = "(?<" name ">" pattern? ")"
            | "(?'" name "'" pattern? ")"
```

Both spellings (angle-bracket and apostrophe) collapse to ONE kind with the name as a
clean string:

```json
{ "type": "atom", "kind": "named_group", "name": "n", "body": <pattern> }
```

`(?<n>a)` and `(?'n'a)` emit identical shapes — the syntactic spelling is not
preserved (match on `kind` + `name`).

## `python_named_group`

```ebnf
python_named_group = "(?P<" name ">" pattern? ")"
```

```json
{ "type": "atom", "kind": "python_named_group", "name": "name", "body": <pattern> }
```

A distinct `kind` from `named_group`, paralleling `python_named_backreference` —
PCRE2 treats `(?P<n>...)` as equivalent to `(?<n>...)`, but tooling that reproduces
the source wants the syntactic origin. Consumers normalizing all name-based groups:
`kind in {"named_group", "python_named_group"}`; `name` carries the name in both.

## `atomic_group`

```ebnf
atomic_group = "(?>" pattern? ")" | "(*atomic:" pattern? ")"
```

Both spellings collapse to one kind:

```json
{ "type": "atom", "kind": "atomic_group", "body": <pattern> }
```

## `branch_reset_group`

```ebnf
branch_reset_group = "(?|" pattern? ")"
```

```json
{ "type": "atom", "kind": "branch_reset_group", "body": <pattern> }
```

The alternation the construct exists for lives inside `body` (its tail carries the
`("|" alternative)*` pairs).

## `lookaround`

```ebnf
lookaround = lookahead_pos | lookahead_neg | lookbehind_pos | lookbehind_neg
           | non_atomic_lookahead_pos | non_atomic_lookbehind_pos | alpha_lookaround
```

The seven syntactic branches collapse to FOUR kinds with a `positive` boolean (plus
the alpha spelling, which keeps its name):

| Form | Emitted atom |
|---|---|
| `(?=...)` | `{"type":"atom","kind":"lookahead","positive":true,"body":<pattern>}` |
| `(?!...)` | `{"type":"atom","kind":"lookahead","positive":false,"body":<pattern>}` |
| `(?<=...)` | `{"type":"atom","kind":"lookbehind","positive":true,"body":<pattern>}` |
| `(?<!...)` | `{"type":"atom","kind":"lookbehind","positive":false,"body":<pattern>}` |
| `(?*...)` | `{"type":"atom","kind":"non_atomic_lookahead","positive":true,"body":<pattern>}` |
| `(?<*...)` | `{"type":"atom","kind":"non_atomic_lookbehind","positive":true,"body":<pattern>}` |
| `(*pla:...)`, `(*nla:...)`, `(*plb:...)`, `(*nlb:...)`, `(*napla:...)`, `(*naplb:...)` + full names | `{"type":"atom","kind":"alpha_lookaround","name":"<alpha-name>","body":<pattern>}` |

PCRE2 only supports positive non-atomic lookarounds; their `positive:true` is emitted
for consumer-code uniformity. The alpha form's `name` is the exact source spelling
(`"nla"`, `"negative_lookahead"`, …) — consumers map it to the semantic equivalent.

**`\K` is rejected inside a lookaround (release `1.1.101`, `REGEX-PCRE2-FIDELITY.4.10`).** As of `1.1.101`, each of the seven lookaround branches consumes its opener through a small **open-marker** rule (`lookahead_pos_open = "(?="`, …, `alpha_lookaround_open = "(*" alpha_lookaround_name ":"`) that opens a `lookaround` semantic scope, and closes it after the body. That scope lets the `keep_out` anchor (`\K`) reject when it is anywhere inside a lookaround body (PCRE2 error 199) — see [the anchors chapter](./examples-anchors.md#k-is-rejected-inside-a-lookaround-release-111101-regex-pcre2-fidelity410). The refactor is **AST-shape-neutral**: the marker's own output is discarded by the parent's `-> {…, body: $2}`, so every accepted lookaround AST is byte-identical to before (the alpha form's `name`/`body` still carry the alpha name and inner pattern).

## `subroutine_call`

```ebnf
subroutine_call = "(?" returned_capture_subroutine ")"
                | "(?" subroutine_target ")"
```

```json
{ "type": "atom", "kind": "subroutine_call", "target": <target> }
```

The `target` object discriminates on its own `kind`:

| Source | `target` |
|---|---|
| `(?&name)` | `{"kind":"named","name":"name"}` |
| `(?P>name)` | `{"kind":"python_named","name":"name"}` |
| `(?R)` | `{"kind":"recursion"}` |
| `(?1)` | `{"kind":"numeric","sign":[],"value":1}` |
| `(?+1)` | `{"kind":"numeric","sign":"+","value":1}` |
| `(?-1)` | `{"kind":"numeric","sign":"-","value":1}` |

`sign` is `[]` (absolute), `"+"`, or `"-"` (relative); `value` is the typed integer.

> **Named-reference resolution (`REGEX-PCRE2-FIDELITY.4.11`, ledger `REGEX-0098` — CLOSED).**
> Since release `1.1.103`, a named subroutine call (or named backreference) whose name is
> not defined by any group in the pattern REJECTS at parse completion (PCRE2 err 115) —
> `(?&zzz)`, `(?P>zzz)`, `\g<zzz>`, `\g'zzz'`, `(?P=zzz)` with no group `zzz` all reject.
> Forward references (`(?&a)(?<a>x)`) remain LEGAL: the check is a whole-input
> `phase: final` deferred obligation against the complete capture-name inventory, so it
> is order-independent. The accepted AST shape is unchanged by the gate.

## `scan_substring_group`

```ebnf
scan_substring_group = "(*" scan_substring_name ":" returned_capture_group_list pattern? ")"
```

```json
{ "type": "atom", "kind": "scan_substring_group", "name": "scs",
  "captures": ["(", {"sign": [], "value": 1}, [], ")"],
  "body": <pattern> }
```

- `name` is the source spelling (`"scs"` or `"scan_substring"`).
- `captures` is the capture-reference list in its raw list carrier (delimiters
  preserved): each referenced capture appears either as a typed signed-number object
  (`{"sign":[],"value":1}` for `(1)`) or as a clean name string (`"n"` for `(<n>)` /
  `('n')`).
- `body` is the scanned pattern.

A **NAMED** capture reference in the list must reference a capture name defined
SOMEWHERE in the pattern — a forward reference (defined later) is legal, an undefined
name REJECTs at parse completion (PCRE2 err 115). This is grammar-owned since
`REGEX-PCRE2-FIDELITY.4.7.a` via a whole-input `phase: final`
`has_fact(regex_defined_capture_name, $name)` gate (the second consumer of the
deferred-obligation primitive after the `.4.11` named backreferences). NUMERIC
references (`(*scs:(N))`) remain validator-owned pending `.4.7.b`/`.4.7.c`. The
accepted AST shape is unchanged by the gate.

## `script_run_group`

```ebnf
script_run_group = "(*" script_run_name ":" pattern? ")"
```

```json
{ "type": "atom", "kind": "script_run_group", "name": "sr", "body": <pattern> }
```

`name` is the source spelling: `"sr"`, `"script_run"`, `"asr"`, or
`"atomic_script_run"`.

## `conditional`

```ebnf
conditional = "(?(" condition ")" yes_branch ("|" no_branch)? ")"
```

```json
{ "type": "atom", "kind": "conditional",
  "condition": <condition>,
  "yes_branch": [<pieces>],
  "no_branch": ["|", [<pieces>]] }
```

- `yes_branch` is the piece array of the yes-concatenation.
- `no_branch` is the optional `("|" no_branch)?` slot: the 2-element
  `["|", [<pieces>]]` pair when present, `[]` when absent (`(?(1)y)`).
- `condition` is a typed value discriminated by its own shape — see the condition
  table in [Anchors, Backreferences, and Misc](rules-misc.md#condition). Summary of
  the forms (all live-verified):

| Source | `condition` |
|---|---|
| `(?(1)…)` | `{"sign":[],"value":1}` (signed-number object; `sign` `"+"`/`"-"` for relative) |
| `(?(<n>)…)` / `(?('n')…)` | `"n"` (clean name string) |
| `(?(name)…)` | `"name"` (bare-name form — same clean string) |
| `(?(R)…)` | `{"kind":"recursion","group":[]}` |
| `(?(R2)…)` | `{"kind":"recursion","group":2}` |
| `(?(R&n)…)` | `{"kind":"recursion_named","name":"n"}` |
| `(?(DEFINE)…)` | `{"kind":"define"}` |
| `(?(VERSION>=10.4)…)` | `{"kind":"version","operator":">=","number":{"major":10,"minor":4}}` |
| `(?(?=x)…)` (assertion) | the assertion's own typed lookaround object, e.g. `{"kind":"lookahead","positive":true,"body":<pattern>}` |
| `(?(?C1)(?=x)…)` (callout-prefixed assertion) | `{"kind":"callout_assertion","callout":{"kind":"callout","arg":1},"assertion":<lookaround object>}` |

## `python_named_backreference`

```ebnf
python_named_backreference = "(?P=" name ")"
```

```json
{ "type": "backreference", "kind": "python_named", "ref": "name" }
```

Note the `type` is `"backreference"` (not `"atom"`) — it joins the `\k...`/`\g...`
backreference family documented in [Atom Subtree](rules-atom.md#backreference).
`(?P=zzz)` for an UNKNOWN group name REJECTS since `1.1.103` (the `.4.11` gate above).

## Auxiliary rules — folded away

The lexical helper rules (`name`, `name_ref`, `braced_name_ref`, `subroutine_ref`,
`signed_digits`, `signed_digits_or_name`, `name_start`, `name_continue`, `brace_ws`,
`sign`) never surface raw in the typed output: every group construct folds them into
the clean fields above (`name: "foo"`, `target: {...}`, `condition: ...`). A name is
always a plain joined string; a signed number is always `{"sign", "value"}` with a
typed integer `value`.

## Walking a `(?P<foo>bar)` example

Exact probe output for `(?P<foo>bar)` (the piece around the atom shown too):

```json
{
  "atom": {
    "type": "atom", "kind": "python_named_group", "name": "foo",
    "body": [[[
      { "atom": "b", "quantifier": [], "type": "piece" },
      { "atom": "a", "quantifier": [], "type": "piece" },
      { "atom": "r", "quantifier": [], "type": "piece" }
    ]], []]
  },
  "quantifier": [],
  "type": "piece"
}
```

Consumer extraction is two field reads:

```rust
fn extract_named_group(atom: &Value) -> Option<(&str, &Value)> {
    let obj = atom.as_object()?;
    match obj.get("kind")?.as_str()? {
        "named_group" | "python_named_group" => {
            Some((obj.get("name")?.as_str()?, obj.get("body")?))
        }
        _ => None,
    }
}
```

## Historical note

Before the atom-subtree campaign (groups/lookarounds typed at slice 23, post-1.1.53;
conditionals, subroutine targets, scan-substring and script-run in follow-up slices)
this family emitted raw delimiter sequences (`["(", <pattern?>, ")"]`,
`["(?<", <name>, ">", <pattern?>, ")"]`, …) and consumers extracted names by walking
`name_start name_continue*` char arrays. Those shapes no longer exist in any released
artifact — migrate to the `kind` dispatch + field reads above. (Release-by-release
record: [Changelog Index](changelog-index.md).)
