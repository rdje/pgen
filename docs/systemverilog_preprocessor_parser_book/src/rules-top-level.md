# Top-Level Rules

This chapter is the per-rule shape reference for the PGEN sv_preprocessor parser. It documents the `systemverilog_preprocessor_file` root, the `pp_item` dispatch, the seven directive shapes, the conditional-compilation tree, the condition / macro-formal / macro-body atom families, and the passthrough lines — grouped by rule family.

> **Status:** SVPP-Slice-1 landed the full grammar typing in one comprehensive batch; the `1.0.2` `SVPP-0001` correctness fix then lifted the inline `(kw_ifdef | kw_ifndef)` alternation into the named `pp_if_keyword` rule. As of parser release `1.0.2` / AST-dump schema version `2` the surface is **66 return annotations across 28 distinct rules** in `grammars/systemverilog_preprocessor.ebnf`. Every shape in this chapter is drawn from the live inventory at `generated/systemverilog_preprocessor_return_annotations.json` (cross-checked against the embedded inventory in `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` — identical content; the contract-embedded copy omits only the cosmetic `raw_text` field, the `(rule, branch_index, annotation_type, normalized_text)` tuples are byte-identical, 66 entries). That artifact, not this prose, is the machine-checkable source of truth.

## How to read this chapter

This is a **curated, grouped** reference — not a raw 64-line dump and not a copy of any IEEE 1800 LRM. For each family it gives the `kind` discriminators and field lists that the parser actually emits, transcribed from each rule's normalized return-annotation text. Where a rule has per-branch typing, the `kind` value names the matched branch; where a rule has a single sequence shape, the named fields are listed directly.

Two conventions appear throughout:

- **Dispatch rules** emit `{kind: "<branch>", body: $N}` (or just `{kind: "<branch>"}` for bodyless branches). Consumers dispatch on `obj["kind"]`.
- **Absent optionals are `[]`, never `null`.** An un-matched optional grammar element (`macro_formals?`, `macro_body?`, `directive_tail?`, the optional comment tail, …) surfaces as the empty array `[]`. This was verified empirically against the live parser — see [The Json Carrier](json-carrier.md).

This grammar is **directive- and line-oriented**. Unlike the VHDL and rtl_frontend grammars, it has **no** operator-precedence cascade and **no** `binop_chain` carrier. `condition_expr` is a flat atom list (`condition_atom+`), not a precedence tree — the `||` / `&&` / `!` / `? :` tokens are carried as individual untyped atoms, not as a parsed expression.

The annotation-language conventions (`$N`, `{field: value}`, `[...]`, string literals) follow `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Entry point

| Profile | Entry rule | Description |
|---|---|---|
| `default` | `systemverilog_preprocessor_file` | The SystemVerilog preprocessor directive source surface. The only sv_preprocessor profile. |

There is no per-grammar convenience function for sv_preprocessor; the stable host surface is the generic `parse_grammar_profile*` family with `GrammarFamily::SystemverilogPreprocessor` + `GrammarProfile::Default` (and the `parse_grammar_profile_named` string overload with `"systemverilog_preprocessor"` / `"default"`). See [Public API Surface](public-api.md).

## `systemverilog_preprocessor_file` (root)

Per `grammars/systemverilog_preprocessor.ebnf`:

```ebnf
systemverilog_preprocessor_file := pp_item*
                                -> {type: "systemverilog_preprocessor_file", items: $1}
```

The annotation produces a typed JSON object at the root of every parse:

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [/* array of pp_item shapes */]
}
```

`items` is the array of typed `pp_item` objects, one per top-level line/directive in source order. Consumers walking the sv_preprocessor AST dispatch on `obj["type"] == "systemverilog_preprocessor_file"` at the root, then iterate `obj["items"]`. This is the only rule that carries a `type` discriminator; every other dispatcher uses `kind`.

## `pp_item` dispatch

`pp_item` is the primary top-level dispatcher — a 10-branch `kind`-tagged shape covering every preprocessor line form:

```ebnf
pp_item := pp_define              -> {kind: "define",              body: $1}
         | pp_undef               -> {kind: "undef",               body: $1}
         | pp_include             -> {kind: "include",             body: $1}
         | pp_timescale           -> {kind: "timescale",           body: $1}
         | pp_default_nettype     -> {kind: "default_nettype",     body: $1}
         | pp_celldefine          -> {kind: "celldefine"}
         | pp_endcelldefine       -> {kind: "endcelldefine"}
         | pp_conditional         -> {kind: "conditional",         body: $1}
         | pp_non_directive_line  -> {kind: "non_directive_line",  body: $1}
         | pp_blank_line          -> {kind: "blank_line"}
```

| `kind` | `body` shape |
|---|---|
| `"define"` | `pp_define` — `{name, formals, body}` |
| `"undef"` | `pp_undef` — `{name, comment}` |
| `"include"` | `pp_include` — `{path, comment}` |
| `"timescale"` | `pp_timescale` — `{unit, precision, comment}` |
| `"default_nettype"` | `pp_default_nettype` — `{nettype, comment}` |
| `"celldefine"` | _(no body — `` `celldefine`` marker)_ |
| `"endcelldefine"` | _(no body — `` `endcelldefine`` marker)_ |
| `"conditional"` | `pp_conditional` — `{if_branch, elsif_branches, else_branch}` |
| `"non_directive_line"` | `pp_non_directive_line` — `{text}` |
| `"blank_line"` | _(no body — blank line)_ |

The `"celldefine"`, `"endcelldefine"`, and `"blank_line"` branches are bodyless (`{kind: ...}` only); every other branch carries a `body`. Note that `pp_celldefine` / `pp_endcelldefine` / `pp_blank_line` are themselves typed rules (`{comment}` / `{comment}` / `{kind: "blank"}` — see below): their `body` is dropped at the `pp_item` dispatch level by the annotation, so a consumer that needs the celldefine comment must re-parse the line or treat the directive as state-only.

## Family: core directives (7 shapes)

```ebnf
pp_define := kw_define macro_name macro_formals? macro_body? newline?
          -> {name: $2, formals: $3, body: $4}
pp_undef := kw_undef macro_name directive_comment_tail newline?
         -> {name: $2, comment: $3}
pp_include := kw_include include_path directive_comment_tail newline?
           -> {path: $2, comment: $3}
pp_timescale := kw_timescale time_literal slash time_literal directive_comment_tail newline?
             -> {unit: $2, precision: $4, comment: $5}
pp_default_nettype := kw_default_nettype nettype_value directive_comment_tail newline?
                   -> {nettype: $2, comment: $3}
pp_celldefine := kw_celldefine directive_comment_tail newline?
              -> {comment: $2}
pp_endcelldefine := kw_endcelldefine directive_comment_tail newline?
                 -> {comment: $2}
```

| Rule | Shape | Notes |
|---|---|---|
| `pp_define` | `{name, formals, body}` | `name` is the un-annotated `macro_name`/`identifier` envelope; `formals` is the typed `macro_formals` or `[]` when absent; `body` is the typed `macro_body` or `[]` when absent. |
| `pp_undef` | `{name, comment}` | `comment` is the optional `directive_comment_tail` (inline trivia + optional line comment). |
| `pp_include` | `{path, comment}` | `path` is the typed `include_path`. |
| `pp_timescale` | `{unit, precision, comment}` | `unit` / `precision` are typed `time_literal` (`{value, unit}`); the `/` separator is consumed but not surfaced. |
| `pp_default_nettype` | `{nettype, comment}` | `nettype` is the typed `nettype_value`. |
| `pp_celldefine` | `{comment}` | (`body` dropped at `pp_item`). |
| `pp_endcelldefine` | `{comment}` | (`body` dropped at `pp_item`). |

### Include path / nettype / time literal

```ebnf
include_path := quoted_string -> {kind: "quoted", text: $1}
              | angle_path    -> {kind: "angle",  text: $1}
nettype_value := identifier -> {kind: "identifier", body: $1}
               | kw_none    -> {kind: "none"}
time_literal := unsigned_number time_unit -> {value: $1, unit: $2}
```

| Rule | Shape |
|---|---|
| `include_path` (2 kinds) | `{kind: "quoted", text}` (`` `include "f.svh"``) / `{kind: "angle", text}` (`` `include <pkg.svh>``). |
| `nettype_value` (2 kinds) | `{kind: "identifier", body}` (e.g. `wire`) / `{kind: "none"}` (the `none` keyword, bodyless). |
| `time_literal` | `{value, unit}` — `value` is the un-annotated `unsigned_number` envelope; `unit` is the un-annotated `time_unit` envelope (`s`/`ms`/`us`/`ns`/`ps`/`fs`). |

## Family: conditional-compilation tree (5 nodes + the pp_if_keyword polarity rule)

```ebnf
pp_conditional := pp_if_branch pp_elsif_branch* pp_else_branch? pp_endif
               -> {if_branch: $1, elsif_branches: $2, else_branch: $3}
pp_if_keyword := kw_ifdef  -> {kind: "ifdef"}
              |  kw_ifndef -> {kind: "ifndef"}
pp_if_branch := pp_if_keyword macro_name directive_tail? newline pp_item*
             -> {keyword: $1, macro: $2, tail: $3, items: $5}
pp_elsif_branch := kw_elsif condition_expr newline pp_item*
                -> {condition: $2, items: $4}
pp_else_branch := kw_else directive_tail? newline pp_item*
               -> {tail: $2, items: $4}
pp_endif := kw_endif directive_tail? newline?
         -> {tail: $2}
```

`pp_if_keyword` is the `1.0.2` `SVPP-0001` correctness fix: the
previously-inline `(kw_ifdef | kw_ifndef)` alternation lifted into a
named 2-branch `kind`-tagged rule (the proven `rtl_const_expr`
RTL-CE-Slice-2 / `systemverilog.ebnf` idiom) so `pp_if_branch`'s bare
`keyword: $1` captures cleanly. Its 2 branches are the +2 entries that
took the surface to 66 annotations / 28 distinct rules.

| Rule | Shape | Notes |
|---|---|---|
| `pp_conditional` | `{if_branch, elsif_branches, else_branch}` | `elsif_branches` is the `pp_elsif_branch*` array (`[]` when none); `else_branch` is `[]` when there is no `` `else``. The terminating `pp_endif` is matched (`$4`) but **not** surfaced in the annotation. |
| `pp_if_keyword` | `{kind: "ifdef"}` (branch 0) / `{kind: "ifndef"}` (branch 1) | The 2-branch `` `ifdef``/`` `ifndef`` polarity discriminator landed by the `1.0.2` `SVPP-0001` fix. Both branches `return_object`. |
| `pp_if_branch` | `{keyword, macro, tail, items}` | `keyword` is `$1`, the typed `pp_if_keyword` polarity object — **`{kind: "ifdef"}` or `{kind: "ifndef"}`** as of the `1.0.2` `SVPP-0001` fix (was the malformed `<invalid_sequence_access>` object at `1.0.1` / schema `1`; see [Conditional Compilation](examples-conditional.md) and [Schema Versioning](schema-versioning.md)). Read the polarity from `if_branch.keyword.kind`. `macro` is the macro-name envelope; `tail` is the optional `directive_tail` (`[]` when absent); `items` is the recursively-nested `pp_item*` body of the branch. |
| `pp_elsif_branch` | `{condition, items}` | `condition` is the typed `condition_expr`; `items` is the nested `pp_item*` body. |
| `pp_else_branch` | `{tail, items}` | `tail` is the optional `directive_tail` (`[]` when absent); `items` is the nested `pp_item*` body. |
| `pp_endif` | `{tail}` | `tail` is the optional `directive_tail` (`[]` when absent). |

The branch bodies (`items`) are themselves `pp_item*` arrays, so the conditional tree nests recursively: a `pp_conditional` can appear inside any branch's `items`.

## Family: condition expression (12-kind atom)

```ebnf
condition_expr := condition_atom+
               -> {atoms: $1}
condition_atom := macro_token_paste -> {kind: "token_paste"}
                | macro_stringize   -> {kind: "stringize"}
                | macro_reference   -> {kind: "macro_reference", body: $1}
                | condition_text    -> {kind: "text",     body: $1}
                | lparen            -> {kind: "lparen"}
                | rparen            -> {kind: "rparen"}
                | comma             -> {kind: "comma"}
                | question          -> {kind: "question"}
                | colon             -> {kind: "colon"}
                | logical_or        -> {kind: "logical_or"}
                | logical_and       -> {kind: "logical_and"}
                | bang              -> {kind: "bang"}
```

| Rule | Shape |
|---|---|
| `condition_expr` | `{atoms}` — `atoms` is the recursive-envelope `condition_atom+` array (at least one). |
| `condition_atom` (12 kinds) | `{kind, body}` for `"macro_reference"` and `"text"`; bare `{kind}` for `"token_paste"`, `"stringize"`, `"lparen"`, `"rparen"`, `"comma"`, `"question"`, `"colon"`, `"logical_or"`, `"logical_and"`, `"bang"`. |

This is a **flat token list**, not a parsed boolean expression. The `` `elsif`` condition is surfaced as an ordered sequence of atoms; the consumer is responsible for any expression evaluation. The `body` of the `"macro_reference"` atom is the un-annotated `macro_reference` (backtick-identifier) envelope; the `body` of the `"text"` atom is the un-annotated `condition_text` envelope.

## Family: macro formals + default values (8-kind atom)

```ebnf
macro_formals := lparen macro_formal (comma macro_formal)* rparen
              -> {first: $2, rest: $3}
macro_formal := macro_name (assign macro_default_value)?
             -> {name: $1, default: $2}
macro_default_value := macro_default_atom+
                    -> {atoms: $1}
macro_default_atom := macro_token_paste -> {kind: "token_paste"}
                    | macro_stringize   -> {kind: "stringize"}
                    | macro_reference   -> {kind: "macro_reference", body: $1}
                    | macro_default_text -> {kind: "text",  body: $1}
                    | lparen            -> {kind: "lparen"}
                    | rparen            -> {kind: "rparen"}
                    | question          -> {kind: "question"}
                    | colon             -> {kind: "colon"}
```

| Rule | Shape | Notes |
|---|---|---|
| `macro_formals` | `{first, rest}` | A `{first, rest}` separated-list shape: `first` is the leading `macro_formal`; `rest` is the recursive-envelope iteration of the `(comma macro_formal)*` tail. This grammar uses the `{first, rest}` wrap — it was **not** flattened to the `[$N, $M::2*]` form some other PGEN grammars adopted; see [Walking the AST](walking-the-ast.md#iterating-first-rest-lists). |
| `macro_formal` | `{name, default}` | `name` is the macro-name envelope; `default` is the typed `macro_default_value` or `[]` when there is no `= <default>`. |
| `macro_default_value` | `{atoms}` | `atoms` is the recursive-envelope `macro_default_atom+` array. |
| `macro_default_atom` (8 kinds) | `{kind, body}` for `"macro_reference"` / `"text"`; bare `{kind}` for `"token_paste"`, `"stringize"`, `"lparen"`, `"rparen"`, `"question"`, `"colon"`. This atom set has **no** `"comma"`, `"logical_or"`, `"logical_and"`, or `"bang"` (commas separate formals, so a default value cannot contain a bare comma). |

## Family: macro body (9-kind fragment)

```ebnf
macro_body := macro_body_fragment+
           -> {fragments: $1}
macro_body_fragment := macro_token_paste -> {kind: "token_paste"}
                     | macro_stringize   -> {kind: "stringize"}
                     | macro_reference   -> {kind: "macro_reference", body: $1}
                     | macro_body_text   -> {kind: "text",   body: $1}
                     | lparen            -> {kind: "lparen"}
                     | rparen            -> {kind: "rparen"}
                     | comma             -> {kind: "comma"}
                     | question          -> {kind: "question"}
                     | colon             -> {kind: "colon"}
```

| Rule | Shape | Notes |
|---|---|---|
| `macro_body` | `{fragments}` | `fragments` is the recursive-envelope `macro_body_fragment+` array (the macro replacement text, tokenized). |
| `macro_body_fragment` (9 kinds) | `{kind, body}` for `"macro_reference"` / `"text"`; bare `{kind}` for `"token_paste"`, `"stringize"`, `"lparen"`, `"rparen"`, `"comma"`, `"question"`, `"colon"`. The macro-body atom set adds `"comma"` (vs the 8-kind `macro_default_atom`) but, like the default-atom set, has no `"logical_or"` / `"logical_and"` / `"bang"`. |

The `"token_paste"` fragment is the SystemVerilog `` `` `` paste operator; `"stringize"` is the `` `" `` stringize operator. Each `"text"` fragment carries the un-annotated `macro_body_text` envelope (a run of literal replacement characters).

## Family: passthrough lines

```ebnf
pp_non_directive_line := non_directive_text newline?
                      -> {text: $1}
pp_blank_line := newline
              -> {kind: "blank"}
```

| Rule | Shape | Notes |
|---|---|---|
| `pp_non_directive_line` | `{text}` | Any source line that is not a preprocessor directive. `text` is the un-annotated `non_directive_text` envelope. |
| `pp_blank_line` | `{kind: "blank"}` | A blank line. At the `pp_item` dispatch level this surfaces as `{kind: "blank_line"}` (the `pp_blank_line` rule's own `{kind: "blank"}` body is dropped by the `pp_item` annotation, exactly like `pp_celldefine` / `pp_endcelldefine`). |

## Atom-set summary

The three atom families differ only in which punctuation/operator branches they admit. All three share `"token_paste"`, `"stringize"`, `"macro_reference"`, `"text"`, `"lparen"`, `"rparen"`, `"question"`, `"colon"`:

| Rule | Branches | Extra branches beyond the shared 8 |
|---|---|---|
| `condition_atom` | 12 | `"comma"`, `"logical_or"`, `"logical_and"`, `"bang"` |
| `macro_body_fragment` | 9 | `"comma"` |
| `macro_default_atom` | 8 | _(none — exactly the shared 8)_ |

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.2` is **66 return annotations across 28 distinct rules** (independently re-counted from the inventory: 66 `annotations` entries, all `annotation_type: "return_object"`, over 28 distinct rule names; the `1.0.2` `SVPP-0001` fix added the 2 `pp_if_keyword` branches / +1 distinct rule, taking 64→66 / 27→28). This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/systemverilog_preprocessor_return_annotations.json` — the live return-annotation inventory (`version: 1`, `grammar: "systemverilog_preprocessor"`, `annotation_count: 66`).
- `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` — the embedded inventory used by the AST shape-contract regression lock (`declared_annotation_inventory.annotations`, 66 entries; identical tuples — the contract-embedded copy omits only the cosmetic `raw_text` field).

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after SVPP-Slice-1 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together.
