# The Json Carrier

`ParseContent::Json(serde_json::Value)` is the variant downstream consumers will spend most of their time inside. This chapter explains where it comes from, when it appears, and how consumers should treat it.

## What it is

A `Json(serde_json::Value)` is the runtime representation of the typed shape that a grammar rule's return annotation produces. It carries any of the six JSON value types:

- `Value::Object(Map<String, Value>)` — for `-> {...}` annotations.
- `Value::Array(Vec<Value>)` — for `-> [...]` annotations.
- `Value::String(String)` — for `-> "..."` annotations.
- `Value::Number(Number)` — for numeric literal annotations and integer-coerced `@transform` matches.
- `Value::Bool(bool)` — for `-> true`/`-> false` annotations.
- `Value::Null` — for `-> null` annotations (added in the slice that introduced typed `counted_quantifier_body` `{min, max:null}` for the unbounded `{n,}` form).

## When it appears

The codegen emits `ParseContent::Json(...)` whenever a rule has an explicit return annotation that lifts the rule's output into a typed shape. The current set of annotated rules in `grammars/regex.ebnf`:

| Rule | Annotation | Json shape produced |
|---|---|---|
| `regex` | `-> {type: "regex", pattern: $1}` | Object `{type, pattern}` |
| `pattern` | `-> $1` | Whatever `alternation` produced (transparent passthrough) |
| `concatenation` | `-> [$1**]` | Array (flat, via `**` flatten-spread) |
| `piece` (branch 0) | `-> $1` | Whatever `piece_quoted_run_quantified` produced |
| `piece` (branch 1) | `-> {type: "piece", atom: $1, quantifier: $2}` | Object `{type, atom, quantifier}` |
| `piece_quoted_run_quantified` | `-> [$2**, {type: "piece", atom: $3, quantifier: $5}]` | Array of piece-objects |
| `quoted_run_inner_piece` | `-> {type: "piece", atom: $1, quantifier: []}` | Object `{type, atom, quantifier:[]}` |
| `counted_quantifier` | `-> $3` | Whatever `counted_quantifier_body` produced (transparent passthrough) |
| `counted_quantifier_body` (branch 0) | `-> {min: $1, max: $3}` | Object `{min, max}` (`{n,m}` form) |
| `counted_quantifier_body` (branch 1) | `-> {min: $1, max: null}` | Object `{min, max:null}` (`{n,}` unbounded form) |
| `counted_quantifier_body` (branch 2) | `-> {min: $1, max: $1}` | Object `{min, max}` (`{n}` form, max == min) |
| `counted_quantifier_body` (branch 3) | `-> {min: 0, max: $3}` | Object `{min:0, max}` (`{,m}` form) |
| `anchor` (branch 0..8) | `-> {type: "anchor", kind: "<name>"}` per branch | Object `{type:"anchor", kind:<name>}` — `kind` ∈ `start_of_line` / `end_of_line` / `start_of_input` / `end_of_input_or_before_last_newline` / `end_of_input` / `word_boundary` / `non_word_boundary` / `match_start` / `keep_out` |
| `posix_word_boundary_alias` (branch 0) | `-> {type: "anchor", kind: "posix_word_start"}` | Same anchor family as the `anchor` rule (kind = `posix_word_start`) |
| `posix_word_boundary_alias` (branch 1) | `-> {type: "anchor", kind: "posix_word_end"}` | Same anchor family as the `anchor` rule (kind = `posix_word_end`) |
| `backreference` (branch 0) | `-> {type: "backreference", kind: "numeric", index: $2}` | Object `{type, kind:"numeric", index:<int>}` |
| `backreference` (branch 1) | `-> {type: "backreference", kind: "named", ref: $2}` | Object `{type, kind:"named", ref:<raw name_ref shape>}` |
| `backreference` (branch 2) | `-> {type: "backreference", kind: "named_braced", ref: $2}` | Object `{type, kind:"named_braced", ref:<raw braced_name_ref shape>}` |
| `backreference` (branch 3) | `-> {type: "backreference", kind: "subroutine", ref: $2}` | Object `{type, kind:"subroutine", ref:<raw subroutine_ref shape>}` |
| `backreference_digits` | `@transform: str::parse::<usize>().unwrap_or(0)` | Number (integer) |
| `name_ref` (branch 0, angle) | `-> $2` | Whatever `name` produced (typed name string) |
| `name_ref` (branch 1, quote) | `-> $2` | Whatever `name` produced (typed name string) |
| `braced_name_ref` | `-> $3` | Whatever `name` produced (typed name string) |
| `name` | regex literal `/(...)/` | Terminal of the matched name string (clean, no chain) |
| `subroutine_ref` (branch 0, braced) | `-> $1` | Whatever `braced_subroutine_ref` produced |
| `subroutine_ref` (branch 1, angle) | `-> $2` | Whatever `signed_digits_or_name` produced (string for name, or `[<sign?>, <int>]` for digits) |
| `subroutine_ref` (branch 2, quote) | `-> $2` | Same as branch 1 |
| `subroutine_ref` (branch 3, signed_digits) | `-> $1` | Whatever `signed_digits` produced (typed `{sign, value}` object) |
| `braced_subroutine_ref` | `-> $3` | Whatever `signed_digits_or_name` produced |
| `signed_digits` | `-> {sign: $1, value: $2}` | Object `{sign:<"+"|"-"|[]>, value:<int>}` |
| `escape` | `-> $2` | Whatever `escape_unit` produced (typed object for annotated branches; raw shape for hex/unicode/octal/property pending) |
| `simple_escape` | `-> {type:"escape", kind:"shorthand", char:$1}` | Object `{type:"escape", kind:"shorthand", char:<char>}` |
| `single_byte_escape` | `-> {type:"escape", kind:"single_byte"}` | Object `{type:"escape", kind:"single_byte"}` |
| `control_escape` | `-> {type:"escape", kind:"control", char:$2}` | Object `{type:"escape", kind:"control", char:<char>}` |
| `hex_escape` (branch 0, short `\xNN`) | `-> {type:"escape", kind:"hex", digits:$2}` | Object `{type:"escape", kind:"hex", digits:<hex-string>}` |
| `hex_escape` (branch 1, braced `\x{...}`) | `-> {type:"escape", kind:"hex", digits:$3}` | Object `{type:"escape", kind:"hex", digits:<hex-string>}` |
| `unicode_escape` | `-> {type:"escape", kind:"unicode", digits:$2}` | Object `{type:"escape", kind:"unicode", digits:<hex-string>}` |
| `hex_digits` | regex literal `/([0-9A-Fa-f]+)/` | Terminal of the matched hex string (was `hex_digit+` chain) |
| `hex_escape_short_payload` | regex literal `/([0-9A-Fa-f]{1,2})/` | Terminal of the matched 1-2 hex digits |
| `octal_escape` (branch 0, braced `\o{...}`) | `-> {type:"escape", kind:"octal", digits:$3}` | Object `{type:"escape", kind:"octal", digits:<octal-string>}` |
| `octal_escape` (branch 1, bare `\NNN` in classes) | `-> {type:"escape", kind:"octal", digits:$1}` | Object `{type:"escape", kind:"octal", digits:<octal-string>}`. Note: at atom-level, bare `\NNN` is shadowed by the `backreference` numeric branch under PEG ordering — pre-existing. |
| `octal_digits` | regex literal `/([0-7]+)/` | Terminal of the matched octal string (was `octal_digit+` chain) |
| `octal_escape_short_payload` | regex literal `/([0-7]{1,3})/` | Terminal of the matched 1-3 octal digits |
| `property_escape` (branch 0, `\p{...}`) | `-> {type:"escape", kind:"property", name:$2, negated:false}` | Object `{type:"escape", kind:"property", name:<str>, negated:false}` |
| `property_escape` (branch 1, `\P{...}`) | `-> {type:"escape", kind:"property", name:$2, negated:true}` | Object `{type:"escape", kind:"property", name:<str>, negated:true}` |
| `property_escape` (branch 2, `\pX`) | `-> {type:"escape", kind:"property", name:$2, negated:false}` | Object `{type:"escape", kind:"property", name:<single-letter>, negated:false}` |
| `property_escape` (branch 3, `\PX`) | `-> {type:"escape", kind:"property", name:$2, negated:true}` | Object `{type:"escape", kind:"property", name:<single-letter>, negated:true}` |
| `prop_name` | regex literal `/([A-Za-z0-9 \t\n\r\f\v_:\-=&^]+)/` | Terminal of the matched property identifier (was `prop_name_chars+` chain) |
| `short_prop_letter` | regex literal `/([CLMNPSZclmnpsz])/` | Terminal of the matched single-letter shorthand (was Or-of-single-chars) |
| `posix_class` | `-> {type: "posix_class", name: $3, negated: $2}` | Object `{type:"posix_class", name:<str>, negated:<true \| []>}` |
| `posix_negation` | `-> true` | Boolean `true` (matched), or `[]` from the un-matched `posix_negation?` slot |
| `quant_base` (branch 0 `*`) | `-> {min: 0, max: null}` | Object `{min:0, max:null}` (unbounded zero-or-more) |
| `quant_base` (branch 1 `+`) | `-> {min: 1, max: null}` | Object `{min:1, max:null}` (unbounded one-or-more) |
| `quant_base` (branch 2 `?`) | `-> {min: 0, max: 1}` | Object `{min:0, max:1}` (zero-or-one) |
| `quant_base` (branch 3 `counted_quantifier`) | `-> $1` | Whatever `counted_quantifier` produced (typed `{min, max}`) |
| `quantifier` | `-> {type: "quantifier", min: $1.min, max: $1.max, greediness: $2}` | Object `{type, min, max, greediness}` |
| `quant_suffix` (branch 0) | `-> "lazy"` | String `"lazy"` |
| `quant_suffix` (branch 1) | `-> "possessive"` | String `"possessive"` |
| `quoted_literal` | `-> {type:"atom", kind:"quoted_literal", body:$2}` | Object `{type:"atom", kind:"quoted_literal", body:<array-of-chars>}`. Array elements are single-char strings; `quoted_literal_escaped_char` produces 2-char strings preserving the `\` and the escaped char. |
| `python_named_backreference` | `-> {type:"backreference", kind:"python_named", ref:$2}` | Object `{type:"backreference", kind:"python_named", ref:<name>}`. PCRE2-equivalent to `\k<name>` for matching; `kind` preserves syntax origin. |
| `comment_group` | `-> {type:"atom", kind:"comment", text:$2}` | Object `{type:"atom", kind:"comment", text:<string>}`. `text` is always a string (empty for `(?#)`). |
| `comment_text` | regex literal `/([^)]*)/` | Terminal of the comment body — any chars except `)` (was `comment_char*` chain). |
| `capturing_group` | `-> {type:"atom", kind:"capturing_group", body:$2}` | Object `{type:"atom", kind:"capturing_group", body:<pattern>}`. `body` is `[[], []]` for `()`. |
| `noncapturing_group` | `-> {type:"atom", kind:"noncapturing_group", body:$2}` | Object `{type:"atom", kind:"noncapturing_group", body:<pattern>}`. |
| `branch_reset_group` | `-> {type:"atom", kind:"branch_reset_group", body:$2}` | Object `{type:"atom", kind:"branch_reset_group", body:<pattern>}`. |
| `atomic_group` (branch 0, `(?>...)`) | `-> {type:"atom", kind:"atomic_group", body:$2}` | Object `{type:"atom", kind:"atomic_group", body:<pattern>}`. |
| `atomic_group` (branch 1, `(*atomic:...)`) | `-> {type:"atom", kind:"atomic_group", body:$2}` | Same kind as branch 0 — PCRE2-equivalent semantics. |
| `named_group` (branch 0, `(?<n>...)`) | `-> {type:"atom", kind:"named_group", name:$2, body:$4}` | Object `{type:"atom", kind:"named_group", name:<str>, body:<pattern>}`. |
| `named_group` (branch 1, `(?'n'...)`) | `-> {type:"atom", kind:"named_group", name:$2, body:$4}` | Same kind as branch 0 — PCRE2-equivalent. |
| `python_named_group` | `-> {type:"atom", kind:"python_named_group", name:$2, body:$4}` | Distinct kind preserves Python-style syntax origin. PCRE2-equivalent to `named_group`. |
| `lookahead_pos` | `-> {type:"atom", kind:"lookahead", positive:true, body:$2}` | Object `{type:"atom", kind:"lookahead", positive:true, body:<pattern>}`. |
| `lookahead_neg` | `-> {type:"atom", kind:"lookahead", positive:false, body:$2}` | Same kind as `_pos`; `positive:false` for negation. |
| `lookbehind_pos` | `-> {type:"atom", kind:"lookbehind", positive:true, body:$2}` | Object `{type:"atom", kind:"lookbehind", positive:true, body:<pattern>}`. |
| `lookbehind_neg` | `-> {type:"atom", kind:"lookbehind", positive:false, body:$2}` | Same kind as `_pos`; `positive:false` for negation. |
| `non_atomic_lookahead_pos` | `-> {type:"atom", kind:"non_atomic_lookahead", positive:true, body:$2}` | Distinct kind; PCRE2 has no negative non-atomic variant. |
| `non_atomic_lookbehind_pos` | `-> {type:"atom", kind:"non_atomic_lookbehind", positive:true, body:$2}` | Distinct kind; PCRE2 has no negative non-atomic variant. |
| `alpha_lookaround` | `-> {type:"atom", kind:"alpha_lookaround", name:$2, body:$4}` | Object `{type:"atom", kind:"alpha_lookaround", name:<alpha_lookaround_name>, body:<pattern>}`. Consumers map `name` to dispatch on semantic equivalent. |
| `inline_modifiers` | `-> {type:"atom", kind:"inline_modifiers", spec:$2}` | Object. `spec` is the raw `modifier_spec` shape (sub-rule typing is a separate slice). `[]` when `modifier_spec?` is un-matched (e.g. `(?)`). |
| `scoped_inline_modifiers` | `-> {type:"atom", kind:"scoped_inline_modifiers", spec:$2, body:$4}` | Object. `body` is the inner pattern (raw). |
| `callout` | `-> {type:"atom", kind:"callout", arg:$2}` | Object. `arg` is `callout_arg`'s typed-int (digits) or string (callout_string) shape. `[]` when un-matched. |
| `directive_verb` | `-> {type:"atom", kind:"directive_verb", body:$2}` | Object. `body` is the raw `directive_body` shape (sub-rule typing is a separate slice). |
| `code_block_plain` | `-> {type:"atom", kind:"code_block", lang:null, content:$2}` | Object. `lang:null` distinguishes plain form. `content` is `code_content` (raw). |
| `code_block_lang` | `-> {type:"atom", kind:"code_block", lang:$2, content:$4}` | Same kind as plain; `lang` is the matched language ident. |
| `scan_substring_group` | `-> {type:"atom", kind:"scan_substring_group", name:$2, captures:$4, body:$5}` | Object. `name` is `"scs"` or `"scan_substring"` (PCRE2-equivalent). `captures` is the raw `returned_capture_group_list` shape. |
| `script_run_group` | `-> {type:"atom", kind:"script_run_group", name:$2, body:$4}` | Object. `name` is `"sr"`/`"script_run"`/`"asr"`/`"atomic_script_run"` (atomic vs non-atomic encoded in name). |
| `subroutine_call` (branch 0, with captures) | `-> {type:"atom", kind:"subroutine_call", target:$2}` | Object. `target` is `returned_capture_subroutine` (target + capture-list). |
| `subroutine_call` (branch 1, plain) | `-> {type:"atom", kind:"subroutine_call", target:$2}` | Same kind; `target` is just `subroutine_target`. Inspect `target` shape to determine syntactic form. |
| `char_class` | `-> {type:"atom", kind:"char_class", negated:$2, initial_close:$3, body:$4}` | Object. `negated`/`initial_close` are `true` matched, `[]` un-matched. `body` is raw class_body shape; inner posix_class/class_range/etc. items typed by earlier slices propagate. |
| `negation` | `-> true` | Boolean `true` (matched), `[]` from un-matched `negation?` slot. |
| `class_initial_close` | `-> true` | Boolean `true` (matched), `[]` from un-matched `class_initial_close?` slot. |
| `conditional` | `-> {type:"atom", kind:"conditional", condition:$2, yes_branch:$4, no_branch:$5}` | Object. `condition` is heterogeneous (typed signed_digits / "DEFINE" / `["R", ...]` / name string). `no_branch` is `[]` (no else) or `["|", <pieces>]` (else present). |
| `conditional_branch` | `-> [$1**]` | Flat array of pieces (parallels `concatenation`). |
| `extended_class` | `-> {type:"atom", kind:"extended_class", body:$2}` | Object. `body` is raw `extended_class_content` shape; sub-rule typing of the recursive set-operation structure is a separate concern. |
| `class_range` | `-> {type:"class_range", start:$1, end:$5}` | Object. `start`/`end` are typed class_atoms (escape / clean string / quoted_class_range_atom). `class_zero_width*` slots (rare PCRE2 `\E`/`\Q\E` markers around the dash) dropped from typed shape; consumers needing them fall back to raw. |
| `quoted_class_literal` | `-> {type:"class_quoted_literal", body:$2}` | Object. `body` is array of `quoted_class_literal_char*` matched chars (parallels `quoted_literal` slice 18). |
| `class_range_escape` | `-> $2` | Transparent passthrough — drops the leading `\` so the typed escape_unit shape (slices 14-17) surfaces directly. Mirrors outer `escape -> $2` (slice 14). |
| `subroutine_target` (branch 0, `&name`) | `-> {kind:"named", name:$2}` | Object. Surfaces inside `subroutine_call.target`. |
| `subroutine_target` (branch 1, `P>name`) | `-> {kind:"python_named", name:$2}` | Distinct kind preserves Python syntax origin (paralleling `python_named_backreference` slice 19). |
| `subroutine_target` (branch 2, `R`) | `-> {kind:"recursion"}` | Bare object; no fields. |
| `subroutine_target` (branch 3, signed_digits) | `-> {kind:"numeric", value:$1.value, sign:$1.sign}` | Inlines signed_digits' typed `{sign, value}` shape via field-access (slice 13). |
| `modifier_spec` (branch 0, `(?^...)`) | `-> {reset:true, seq:$2}` | Object. `reset:true` distinguishes the reset-all-flags form. `seq` is raw modifier_seq shape. |
| `modifier_spec` (branch 1, plain) | `-> {reset:false, seq:$1}` | Object. `reset:false` for the plain form. |
| `define_condition` | `-> {kind:"define"}` | Object. Disambiguates `(?(DEFINE)...)` from a name-condition string `"DEFINE"`. |
| `version_condition` | `-> {kind:"version", operator:$2, number:$3}` | Object. `operator` is `">="`/`"="` literal; `number` is raw `version_number` shape (`digits ("." digits)?`). |
| `recursion_condition` (branch 0, `R`/`R<digits>`) | `-> {kind:"recursion", group:$2}` | Object. `group` is `[]` (no number) or typed int (numbered ref). |
| `recursion_condition` (branch 1, `R&name`) | `-> {kind:"recursion_named", name:$2}` | Object. `name` is the named-recursion reference. |
| `callout_backtick_string` | `-> {quote:"backtick", payload:$2}` | Object. `quote` is text-label discriminator; `payload` is inner string. |
| `callout_single_string` | `-> {quote:"single", payload:$2}` | Same shape; `quote:"single"`. |
| `callout_double_string` | `-> {quote:"double", payload:$2}` | Same shape; `quote:"double"`. Text-label used because bootstrap annotation parser rejects `"\""` escape. |
| `callout_caret_string` | `-> {quote:"caret", payload:$2}` | Same shape; `quote:"caret"`. |
| `callout_percent_string` | `-> {quote:"percent", payload:$2}` | Same shape; `quote:"percent"`. |
| `callout_hash_string` | `-> {quote:"hash", payload:$2}` | Same shape; `quote:"hash"`. |
| `callout_dollar_string` | `-> {quote:"dollar", payload:$2}` | Same shape; `quote:"dollar"`. |
| `callout_brace_string` | `-> {quote:"brace", payload:$2}` | Same shape; `quote:"brace"`. Asymmetric delimiters (`{` opening, `}` closing) — the `"brace"` label captures both. |
| `directive_named` | `-> {kind:"named", name:$1, payload:$2}` | Object. `name` is a clean string (after slice 34's regex-literal rewrite of `directive_name`); `payload` is raw `directive_payload_suffix?` shape. |
| `directive_mark_shorthand` | `-> {kind:"mark_shorthand", payload:$2}` | Object. `payload` is raw `directive_payload_simple?` shape. |
| `directive_name` | regex literal `/([A-Za-z][A-Za-z0-9_\-]*)/` | Terminal of the matched verb name (was `directive_name_start directive_name_continue*` chain). |
| `directive_payload_suffix` (branch 0, `:`) | `-> {separator:":", value:$2}` | Object. `value` is `directive_payload_simple` (clean string after slice 35 regex-literal rewrite); `[]` when un-matched. |
| `directive_payload_suffix` (branch 1, `=`) | `-> {separator:"=", value:$2}` | Same shape, `separator:"="`. |
| `directive_payload_simple` | regex literal `/([^)]*)/` | Terminal of the payload body (any char except `)` — matches the verb-closing). Was `directive_payload_char*` chain. |
| `condition_assertion` (branch 0, `?=`) | `-> {kind:"lookahead", positive:true, body:$2}` | Object. Surfaces inside `conditional.condition`. |
| `condition_assertion` (branch 1, `?!`) | `-> {kind:"lookahead", positive:false, body:$2}` | Same kind, `positive:false`. |
| `condition_assertion` (branch 2, `?<=`) | `-> {kind:"lookbehind", positive:true, body:$2}` | Object. |
| `condition_assertion` (branch 3, `?<!`) | `-> {kind:"lookbehind", positive:false, body:$2}` | Object. |
| `alpha_condition_assertion` | `-> {kind:"alpha_lookaround", name:$2, body:$4}` | Object. Parallels slice 23's atom-level `alpha_lookaround`. |
| `condition_callout` | `-> {kind:"callout", arg:$2}` | Object. Inside-condition variant of atom-level callout (without `(?` prefix); same `{kind, arg}` shape. |
| `condition_callout_assertion` | `-> {kind:"callout_assertion", callout:$1, assertion:$3}` | Object. Wraps the typed callout + assertion sub-shapes. |
| `version_number` (branch 0, `digits "." digits`) | `-> {major:$1, minor:$3}` | Object. Both fields are typed ints (digits @transform). |
| `version_number` (branch 1, `digits`) | `-> {major:$1, minor:null}` | Object. `minor:null` for absent-minor case. |
| `returned_capture_subroutine` | `-> {subroutine:$1, captures:$2}` | Object. `subroutine` is the typed `subroutine_target` (slice 30); `captures` is the raw `returned_capture_group_list` shape pending follow-up flattening. Inner field named `subroutine` (not `target`) to avoid `target.target.kind` collision with the outer `subroutine_call.target`. |
| `digits` | `@transform: str::parse::<usize>().unwrap_or(0)` | Number (integer) |
| `posix_class` | `-> $1` | Whatever the matched element produced |

Rules NOT in this list produce non-`Json` content (`Sequence`, `Quantified`, `Terminal`, `Alternative`) — they inherit the legacy recursive-envelope shape pending future annotation slices.

## How a consumer should treat it

### Pattern matching

```rust
use serde_json::Value;

fn walk(node: &ParseNode) {
    match &node.content {
        ParseContent::Json(value) => walk_json(value),
        ParseContent::Sequence(nodes) => nodes.iter().for_each(walk),
        ParseContent::Alternative(boxed) => walk(boxed),
        ParseContent::Quantified(nodes, _marker) => nodes.iter().for_each(walk),
        ParseContent::Terminal(s) => leaf_terminal(s),
        ParseContent::TransformedTerminal(s) => leaf_terminal(s),
    }
}

fn walk_json(value: &Value) {
    match value {
        Value::Object(map) => {
            // Most-common shape — inspect "type" discriminator.
            match map.get("type").and_then(|v| v.as_str()) {
                Some("regex") => /* {pattern: ...} */,
                Some("piece") => /* {atom, quantifier} */,
                Some(other) => /* unknown — log */,
                None => /* untagged object — see per-rule shapes */,
            }
        }
        Value::Array(items) => /* iterate */,
        Value::String(s) => /* leaf string, e.g. "lazy" */,
        Value::Number(n) => /* integer or float */,
        Value::Bool(b) => /* true/false */,
        Value::Null => /* explicit absence — e.g. unbounded max */,
    }
}
```

### Discriminator convention

Most object-shaped `Json` values carry a `"type"` field as their discriminator. The current discriminators in regex output:

- `"regex"` — emitted by the `regex` rule.
- `"piece"` — emitted by all piece-emitting rules.

Counted-quantifier bodies (the typed `{min, max}` shape) and quant_suffix outputs do NOT have `"type"` discriminators because their shape is unambiguous from context (they always appear inside a `quantifier` slot).

When walking, treat `"type"` as the canonical discriminator when present. Don't rely on field-presence as a proxy for type — that will silently break when a future shape adds optional fields.

### Mixed Json + Sequence in the same level

The `concatenation` rule's `[$1**]` flatten-spread produces a `Sequence` of `ParseNode`s, where each child node may have its own `Json(piece_obj)` content. So when walking pattern's contents, you'll see:

```text
ParseNode { rule_name: "concatenation", content: Sequence([
  ParseNode { rule_name: "...", content: Json({piece}) },
  ParseNode { rule_name: "...", content: Json({piece}) },
  ParseNode { rule_name: "...", content: Json({piece}) },
])}
```

(The synthetic rule_names of children aren't important — they're codegen artifacts.) When you `to_json_value()` this, you get a flat array of piece-objects, which is what the regex JSON dump shows.

For consumers walking the typed shape via `to_json_value()`, the Sequence wrapper is transparent — you just see the array. For consumers walking the ParseNode tree, you do have to descend through the Sequence to get to each piece's Json content.

## Why typed-Json was chosen over typed Rust enums

Two reasons:

1. **Annotation flexibility.** The annotation language can produce arbitrary JSON shapes. Typed Rust enums would have required generating one Rust type per shape, which is intractable for grammars with rich return annotations.
2. **Consumer ergonomics.** Most downstream consumers either serialize to JSON anyway or want to inspect by `as_str()` / `as_object()` etc. — `serde_json::Value` is the natural ergonomic choice.

The downside is that consumers don't get compile-time type checking on the shape — they have to validate at runtime. For the regex grammar, the per-rule shape reference chapters serve as the de-facto schema documentation.

## Why the Json variant exists at all (vs just emitting the inner Value)

The `ParseContent` enum has to carry several shapes (raw `Sequence`, recursive `Alternative`, etc.). `Json` is the variant that says "this content is a fully-typed value, not a recursive AST shape." Wrapping it in a `Json` variant lets the same `ParseContent` enum carry both shapes. Consumers that want to flatten everything to JSON call `to_json_value()`.

## Stability

The set of `serde_json::Value` shapes a given rule emits is documented per-rule and per-release. The `Json` variant itself is stable across PGEN 1.1.x — once a rule is annotated, that rule's annotation is part of the contract. Removing or substantially changing the shape requires a contract version bump.
