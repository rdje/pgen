# Top-Level Rules

This chapter is the per-rule shape reference for the PGEN sv_preprocessor parser. It documents the `systemverilog_preprocessor_file` root, the `pp_item` dispatch, the fifteen directive shapes (every clause-22 directive since release `1.0.8`), the conditional-compilation tree, the condition / macro-formal / macro-body atom families, and the passthrough lines — grouped by rule family.

> **Status:** release `1.0.8` (AST-dump schema `6`) derives every clause-22 directive, a line that opens with a macro usage, the backslash continuation and the `` `include `` macro path — see [Schema Versioning](schema-versioning.md). Earlier: SVPP-Slice-1 landed the full grammar typing in one comprehensive batch; the `1.0.2` `SVPP-0001` correctness fix then lifted the inline `(kw_ifdef | kw_ifndef)` alternation into the named `pp_if_keyword` rule; the `1.0.3` POST-SV-AUDIT Category-A fix corrected `macro_formals` from the raw `{first, rest}` envelope to the clean `[$2, $3::2*]` list; `1.0.5`/schema `4` added `condition_text -> $text`; and the `1.0.7` `SVPP-0004` macro-default balanced-parentheses correction (schema `5`) added `macro_default_paren_group` + `macro_default_group_atom`. As of parser release `1.0.7` / AST-dump schema version `5` the surface is **68 return annotations across 31 distinct rules** in `grammars/systemverilog_preprocessor.ebnf` (65 `return_object` + 1 `return_array` [`macro_formals`] + 2 `return_scalar` [`condition_text`, `macro_default_group_atom`]). Every shape in this chapter is drawn from the live inventory at `generated/systemverilog_preprocessor_return_annotations.json` (cross-checked against the embedded inventory in `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` — identical content; the contract-embedded copy omits only the cosmetic `raw_text` field, the `(rule, branch_index, annotation_type, normalized_text)` tuples are byte-identical, 68 entries). That artifact, not this prose, is the machine-checkable source of truth.

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

`pp_item` is the primary top-level dispatcher — an 18-branch `kind`-tagged shape covering every preprocessor line form (10 branches through `1.0.7`; release `1.0.8` added the eight kinds of the seven clause-22 directives the grammar had not derived — `SV-CORPUS-GRAD.13c.2v.1`):

```ebnf
pp_item := pp_define              -> {kind: "define",              body: $1}
         | pp_undef               -> {kind: "undef",               body: $1}
         | pp_include             -> {kind: "include",             body: $1}
         | pp_timescale           -> {kind: "timescale",           body: $1}
         | pp_default_nettype     -> {kind: "default_nettype",     body: $1}
         | pp_celldefine          -> {kind: "celldefine"}
         | pp_endcelldefine       -> {kind: "endcelldefine"}
         | pp_resetall            -> {kind: "resetall",            body: $1}
         | pp_undefineall         -> {kind: "undefineall",         body: $1}
         | pp_unconnected_drive   -> {kind: "unconnected_drive",   body: $1}
         | pp_nounconnected_drive -> {kind: "nounconnected_drive", body: $1}
         | pp_pragma              -> {kind: "pragma",              body: $1}
         | pp_line                -> {kind: "line",                body: $1}
         | pp_begin_keywords      -> {kind: "begin_keywords",      body: $1}
         | pp_end_keywords        -> {kind: "end_keywords",        body: $1}
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
| `"resetall"` | `pp_resetall` — `{comment}` (`1.0.8`) |
| `"undefineall"` | `pp_undefineall` — `{comment}` (`1.0.8`) |
| `"unconnected_drive"` | `pp_unconnected_drive` — `{drive, comment}`, `drive` `{kind: "pull0"}` / `{kind: "pull1"}` (`1.0.8`) |
| `"nounconnected_drive"` | `pp_nounconnected_drive` — `{comment}` (`1.0.8`) |
| `"pragma"` | `pp_pragma` — `{name, expressions, comment}` (`1.0.8`) |
| `"line"` | `pp_line` — `{number, filename, level}` (`1.0.8`) |
| `"begin_keywords"` | `pp_begin_keywords` — `{version, comment}` (`1.0.8`) |
| `"end_keywords"` | `pp_end_keywords` — `{comment}` (`1.0.8`) |
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
pp_include := kw_include include_path directive_comment_tail line_end
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
| `pp_define` | `{name, formals, body}` | `name` is the un-annotated `macro_name`/`identifier` envelope; `formals` is the typed `macro_formals` — a clean flat `macro_formal[]` list as of `1.0.3` / schema `3` (the POST-SV-AUDIT Category-A `[$2, $3::2*]` extraction-spread; was the raw `{first, rest}` envelope at ≤ `1.0.2` / schema `2`) — or `[]` when absent; `body` is the typed `macro_body` or `[]` when absent. |
| `pp_undef` | `{name, comment}` | `comment` is the optional `directive_comment_tail` (inline trivia + optional line comment). |
| `pp_include` | `{path, comment}` | `path` is the typed `include_path`. |
| `pp_timescale` | `{unit, precision, comment}` | `unit` / `precision` are typed `time_literal` (`{value, unit}`); the `/` separator is consumed but not surfaced. |
| `pp_default_nettype` | `{nettype, comment}` | `nettype` is the typed `nettype_value`. |
| `pp_celldefine` | `{comment}` | (`body` dropped at `pp_item`). |
| `pp_endcelldefine` | `{comment}` | (`body` dropped at `pp_item`). |

⛔ **`` `include `` must end its line** since `1.0.8` (`pp_include := … directive_comment_tail line_end`): IEEE 1800 §22.4, both editions, *"Only white space or a comment may appear on the same line as the `` `include `` compiler directive"* — through `1.0.7` `` `include "a.sv" x `` was accepted, the `x` re-read as a second, passthrough item (`SV-CORPUS-GRAD.13c.2v.4`, ledger `SVPP-0007`). Every other fixed-end directive keeps its optional `newline?`: IEEE 1800-2023 §22.2 lets *"each directive whose syntax shows a defined end to the directive … be followed by another valid language element on the same line"*, which the tree records as a following `non_directive_line` item.

### Include path / nettype / time literal

```ebnf
include_path := quoted_string       -> {kind: "quoted", text: $1}
              | angle_path          -> {kind: "angle",  text: $1}
              | include_macro_usage -> {kind: "macro",  body: $1}
nettype_value := identifier -> {kind: "identifier", body: $1}
               | kw_none    -> {kind: "none"}
time_literal := unsigned_number time_unit -> {value: $1, unit: $2}
```

| Rule | Shape |
|---|---|
| `include_path` (3 kinds) | `{kind: "quoted", text}` (`` `include "f.svh"``) / `{kind: "angle", text}` (`` `include <pkg.svh>``) / `{kind: "macro", body}` (`` `include `home(myfile)`` — IEEE 1800 §22.5.1, both editions: *"The `` `include `` directive can be followed by a macro, instead of a string literal"*; `body` is `{name, arguments}`, `name` the macro's backtick-identifier envelope, `arguments` the balanced `( … )` span as one string or `[]` when absent; a directive keyword is refused there — `` `include `else ``; since `1.0.8`, `SV-CORPUS-GRAD.13c.2v.7`). |
| `nettype_value` (2 kinds) | `{kind: "identifier", body}` (e.g. `wire`) / `{kind: "none"}` (the `none` keyword, bodyless). |
| `time_literal` | `{value, unit}` — `value` is the un-annotated `unsigned_number` envelope; `unit` is the un-annotated `time_unit` envelope (`s`/`ms`/`us`/`ns`/`ps`/`fs`). |

## Family: the clause-22 directives added in `1.0.8` (8 shapes)

Release `1.0.8` (`SV-CORPUS-GRAD.13c.2v.1`) derives the seven IEEE 1800 clause-22 directives the grammar had refused — measured over the SystemVerilog corpus, `` `begin_keywords `` stopped 93 files and `` `resetall `` 69 at release `1.0.7`. Each kind carries its body (the `pp_undef` precedent), so no comment is dropped at the dispatch:

```ebnf
pp_resetall            := kw_resetall directive_comment_tail newline?             -> {comment: $2}
pp_undefineall         := kw_undefineall directive_comment_tail newline?          -> {comment: $2}
pp_unconnected_drive   := kw_unconnected_drive pull_drive directive_comment_tail newline?
                       -> {drive: $2, comment: $3}
pp_nounconnected_drive := kw_nounconnected_drive directive_comment_tail newline?  -> {comment: $2}
pp_pragma              := kw_pragma pragma_name pragma_expression_list? directive_comment_tail line_end
                       -> {name: $2, expressions: $3, comment: $4}
pp_line                := kw_line line_number line_filename line_level line_terminator
                       -> {number: $2, filename: $3, level: $4}
pp_begin_keywords      := kw_begin_keywords version_specifier directive_comment_tail newline?
                       -> {version: $2, comment: $3}
pp_end_keywords        := kw_end_keywords directive_comment_tail newline?         -> {comment: $2}
```

| Rule | Clause | Shape | What it accepts — and refuses |
|---|---|---|---|
| `pp_resetall` | §22.3 | `{comment}` | `` `resetall `` with an optional comment |
| `pp_undefineall` | §22.5.3 (*"takes no arguments"*) | `{comment}` | `` `undefineall `` |
| `pp_unconnected_drive` | §22.9 | `{drive, comment}` | `` `unconnected_drive pull0 `` / `pull1` — `drive` is `{kind: "pull0"}` / `{kind: "pull1"}`; a bare directive or `pull2` is refused |
| `pp_nounconnected_drive` | §22.9 | `{comment}` | `` `nounconnected_drive `` |
| `pp_pragma` | §22.11, Syntax 22-8 | `{name, expressions, comment}` | `name` the `pragma_name` envelope; `expressions` the comma-separated `pragma_expression` list (`[]` when absent). ⛔ Nothing may follow on the line (§22.2 counts `` `pragma `` among the directives *"that include an indeterminate amount of text"*): `` `pragma p a, `` is refused |
| `pp_line` | §22.12, Syntax 22-9 | `{number, filename, level}` | `` `line 5 "f.sv" 0 ``. ⛔ *"only white space may appear on the same line … Comments are not allowed"*: the separators are spaces and tabs only, and a comment, a missing parameter, a level other than `0`/`1`/`2` or a number that is not a positive integer is refused |
| `pp_begin_keywords` | §22.14, Syntax 22-10 | `{version, comment}` | the nine `version_specifier`s of IEEE 1800-2023's list (1800-2017's eight plus `"1800-2023"`); any other text — `"VAMS-2.3"`, an unquoted spelling — is refused. The per-edition admission is the expansion stage's (`W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION`) |
| `pp_end_keywords` | §22.14 | `{comment}` | `` `end_keywords `` |

The `pragma` value grammar follows Syntax 22-8 exactly:

```ebnf
pragma_expression_list := pragma_expression (comma pragma_expression)*  -> [$1, $2::2*]
pragma_expression := pragma_keyword assign pragma_value -> {kind: "keyword_value", keyword: $1, value: $3}
                   | pragma_keyword                     -> {kind: "keyword", keyword: $1}
                   | pragma_value                       -> {kind: "value", value: $1}
pragma_value := pragma_value_list -> {kind: "list", expressions: $1}
              | pragma_number     -> {kind: "number", text: $1}
              | quoted_string     -> {kind: "string", text: $1}
              | identifier        -> {kind: "identifier", name: $1}
```

So `` `pragma protect encoding = (enctype = "BASE64", line_length = 76), begin `` reads as `name` `protect` and two expressions: a `keyword_value` whose value is a `list` of two `keyword_value`s (a `string` and a `number`), then the bare `keyword` `begin`. `pragma_number` is A.8.7's `number` — a real (`3.5e-2`), a sized or unsized based integral number (`4'b10_1x`, `8 'sd 255`, `'hFF`) or an unsigned decimal.

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
took the surface to 66 annotations / 28 distinct rules at release `1.0.2`
(now 68 / 31 at release `1.0.7` / schema `5`, after `condition_text -> $text`
and the `SVPP-0004` `macro_default_paren_group` / `macro_default_group_atom` additions).

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

## Family: macro formals + default values (7-kind atom + the balanced group)

```ebnf
macro_formals := formals_open macro_formal (comma macro_formal)* rparen
              -> [$2, $3::2*]
formals_open := /\(/
macro_formal := macro_name (assign macro_default_value)?
             -> {name: $1, default: $2}
macro_default_value := macro_default_atom+
                    -> {atoms: $1}
macro_default_atom := macro_token_paste -> {kind: "token_paste"}
                    | macro_stringize   -> {kind: "stringize"}
                    | macro_reference   -> {kind: "macro_reference", body: $1}
                    | macro_default_text -> {kind: "text",  body: $1}
                    | macro_default_paren_group
                    | question          -> {kind: "question"}
                    | colon             -> {kind: "colon"}
macro_default_paren_group := lparen macro_default_group_atom* rparen
                          -> {kind: "paren_group", atoms: $2}
macro_default_group_atom := macro_default_atom -> $1
                         | comma -> {kind: "comma"}
```

| Rule | Shape | Notes |
|---|---|---|
| `macro_formals` | `[macro_formal, …]` | A **clean flat list** of typed `macro_formal` objects, in source order — the canonical `[$2, $3::2*]` extraction-spread (`object_properties` idiom), `annotation_type: "return_array"`. At ≤ release `1.0.2` / schema `2` this was the raw `{first, rest}` separator envelope — a Category-A raw-envelope misuse corrected in `1.0.3` / schema `3` (POST-SV-AUDIT.2.1, `PGEN-POST-SV-AUDIT-0002`). See [Walking the AST](walking-the-ast.md#iterating-the-macro_formals-list) and [Schema Versioning](schema-versioning.md). ⛔ **`formals_open` skips nothing** (since `1.0.8`, ledger `SVPP-0008`): IEEE 1800-2017 §22.5.1 *"The left parenthesis shall follow the text macro name immediately, with no space in between"*. So `` `define M(x) y `` is function-like (`formals: [x]`), and `` `define M (x) y `` is OBJECT-like — `formals: []`, the body fragments `lparen`, `text "x"`, `rparen`, `text " y"`. Through `1.0.7` both read as function-like. |
| `macro_formal` | `{name, default}` | `name` is the macro-name envelope; `default` is the typed `macro_default_value` or `[]` when there is no `= <default>`. |
| `macro_default_value` | `{atoms}` | `atoms` is the recursive-envelope `macro_default_atom+` array. |
| `macro_default_atom` (7 kinds + the group) | `{kind, body}` for `"macro_reference"` / `"text"`; bare `{kind}` for `"token_paste"`, `"stringize"`, `"question"`, `"colon"`; `{kind: "paren_group", atoms}` for a balanced `( … )`, inside which (and only there) a comma surfaces as `{kind: "comma"}` (since `1.0.7`, ledger `SVPP-0004`; §22.5.1: a default may contain a comma or right parenthesis only inside *"a balanced pair of parentheses"*). A `"text"` atom keeps a string literal whole since `1.0.8` (`` `define M(a="x,y") `` — the same sentence allows a comma inside *"double quotes"*). |

## Family: macro body (11-kind fragment)

```ebnf
macro_body := macro_body_fragment+
           -> {fragments: $1}
macro_body_fragment := macro_line_continuation -> {kind: "line_continuation"}
                     | macro_token_paste -> {kind: "token_paste"}
                     | macro_stringize   -> {kind: "stringize"}
                     | macro_escaped_quote -> {kind: "escaped_quote"}
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
| `macro_body_fragment` (11 kinds) | `{kind, body}` for `"macro_reference"` / `"text"`; bare `{kind}` for `"line_continuation"`, `"token_paste"`, `"stringize"`, `"escaped_quote"`, `"lparen"`, `"rparen"`, `"comma"`, `"question"`, `"colon"`. The macro-body atom set adds `"comma"`, `"line_continuation"` and `"escaped_quote"` (vs the 8-kind `macro_default_atom`) but, like the default-atom set, has no `"logical_or"` / `"logical_and"` / `"bang"`. |

The `"token_paste"` fragment is the SystemVerilog `` `` `` paste operator; `"stringize"` is the `` `" `` stringize operator; `"escaped_quote"` is `` `\`" `` (§22.5.1: *"The `` `define `` macro text can also include `` `" ``, `` `\`" ``, and ``` `` ```"*; `1.0.8`). Each `"text"` fragment carries the un-annotated `macro_body_text` envelope (a run of literal replacement characters).

⭐ **Three changes in `1.0.8` a consumer of the fragments sees** (schema `5` → `6`):

- **`"line_continuation"`** — a backslash immediately before the newline continues the macro text on the next line (§22.5.1). Through `1.0.7` the body ENDED at the first newline: `` `define M(a) a + \ `` + `  b` gave the text `"a + \"` and a separate `non_directive_line` item for `  b` (ledger `SVPP-0006`). Now the fragments run `text "a + "`, `line_continuation`, `text "b"`, and no stray item follows. A backslash followed by white space is not a continuation.
- **A plain string literal is ONE text atom** — §22.5.1 *"Macro substitution and argument substitution shall not occur within string literals"*: `$display("got `%b`, (%0d)", v)` keeps `"got `%b`, (%0d)"` whole, where the run used to stop at the backtick (refusing the define) or split at `(`, `,`, `:`. `` `define BACKTICK "`" `` no longer reads its `` `" `` as a stringize marker.
- **A `//` comment is ONE text atom**, up to — not including — a continuation backslash (IEEE 1800-2023: *"If a one-line comment is followed by a backslash and newline character, the comment ends before the backslash and the macro text continues on the next line"*).

## Family: passthrough lines

```ebnf
pp_non_directive_line := !directive_keyword non_directive_text newline?
                      -> {text: $2}
pp_blank_line := newline
              -> {kind: "blank"}
```

| Rule | Shape | Notes |
|---|---|---|
| `pp_non_directive_line` | `{text}` | Any source line that is not a preprocessor directive — since `1.0.8` including a line that OPENS with a text macro usage (`` `uvm_object_utils(c) ``, `` `ASSERT(…) ``, `` `__FILE__ ``; §22.5.1 Syntax 22-3, §22.13). `text` is the un-annotated `non_directive_text` envelope. ⛔ `!directive_keyword` — the twenty §22.1 directive keywords — keeps a MALFORMED directive refused: `` `timescale 1ns `` or a stray `` `else `` is not re-read as a macro-usage line. |
| `pp_blank_line` | `{kind: "blank"}` | A blank line. At the `pp_item` dispatch level this surfaces as `{kind: "blank_line"}` (the `pp_blank_line` rule's own `{kind: "blank"}` body is dropped by the `pp_item` annotation, exactly like `pp_celldefine` / `pp_endcelldefine`). |

## Atom-set summary

The three atom families differ only in which punctuation/operator branches they admit. All three share `"token_paste"`, `"stringize"`, `"macro_reference"`, `"text"`, `"lparen"`, `"rparen"`, `"question"`, `"colon"`:

| Rule | Branches | Extra branches beyond the shared 8 |
|---|---|---|
| `condition_atom` | 12 | `"comma"`, `"logical_or"`, `"logical_and"`, `"bang"` |
| `macro_body_fragment` | 11 | `"comma"`, `"line_continuation"`, `"escaped_quote"` |
| `macro_default_atom` | 8 | _(none — exactly the shared 8)_ |

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.8` is **100 return annotations across 46 distinct rules** — **94 `annotation_type: "return_object"` + 4 `return_scalar` + 2 `return_array`** (the arrays are `macro_formals` and, since `1.0.8`, `pragma_expression_list`; the scalars are `condition_text -> $text`, `macro_default_group_atom -> $1`, and, since `1.0.8`, `macro_usage_arguments -> $text` and `pragma_value_list -> $2`). The history: 64 / 27 at SVPP-Slice-1, 66 / 28 after `1.0.2`'s `pp_if_keyword`, 67 / 29 at `1.0.5`, 68 / 31 at `1.0.7`, and **+32 / +15 at `1.0.8`** — the eight new `pp_item` branches, the eight directive rules and their `pull_drive` / `pragma_*` value grammar, `include_macro_usage`, and the two new `macro_body_fragment` kinds. This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/systemverilog_preprocessor_return_annotations.json` — the live return-annotation inventory (`version: 1`, `grammar: "systemverilog_preprocessor"`, `annotation_count: 100`).
- `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` — the embedded inventory used by the AST shape-contract regression lock (`declared_annotation_inventory.annotations`, 100 entries; identical tuples — the contract-embedded copy omits only the cosmetic `raw_text` field), beside thirteen samples whose JSON-pointer claims pin the trees `1.0.8` changed or added.

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after SVPP-Slice-1 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together.
