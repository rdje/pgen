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

## Family: conditional-compilation tree (release `1.0.9`)

The `pp_item` `"conditional"` branch carries a `body` that is the typed `pp_conditional` object, the root of the
`` `ifdef`` / `` `ifndef`` … `` `elsif`` … `` `else`` … `` `endif`` tree. Per `grammars/systemverilog_preprocessor.ebnf`
(release `1.0.9`):

```ebnf
pp_conditional  := pp_if_branch pp_elsif_branch* pp_else_branch? pp_endif
                -> {if_branch: $1, elsif_branches: $2, else_branch: $3}
pp_if_keyword   := kw_ifdef  -> {kind: "ifdef"}
                 | kw_ifndef -> {kind: "ifndef"}
pp_if_branch    := pp_if_keyword if_condition conditional_tail? newline? pp_group_item*
                -> {keyword: $1, macro: $2, tail: $3, items: $5}
pp_elsif_branch := kw_elsif if_condition conditional_tail? newline? pp_group_item*
                -> {macro: $2, tail: $3, items: $5}
pp_else_branch  := kw_else conditional_tail? newline? pp_group_item*
                -> {tail: $2, items: $4}
pp_endif        := kw_endif conditional_tail? newline?
                -> {tail: $2}
@branch_policy: ordered
pp_group_item   := pp_item
                 | pp_malformed_directive -> {kind: "malformed_directive", body: $1}
```

| field | holds |
|---|---|
| `if_branch.keyword` / `.macro` | the polarity (`{kind: "ifdef"}` / `{kind: "ifndef"}`) and the CONDITION (below) |
| `elsif_branches[i].macro` | the `` `elsif `` condition — the same three forms (schema `7`: through schema `6` this was `condition: {atoms}`, a run to the line end) |
| `.tail` (if / elsif / else / endif) | a trailing `//` comment on the directive's line (`[[" "], "// FOO"]`), or `[]`. Any other text after the directive on its line is the group's FIRST ITEM — 1800-2023 §22.6: the block of text ends at the next `` `elsif `` / `` `else `` / `` `endif ``, and *"White space is not considered when determining termination"*. A conditional on one line (`` `ifdef A wire w; `endif ``) is one `conditional` whose `if_branch.items[0]` is the text. |
| `.items` | the group: `pp_item` kinds, plus `{kind: "malformed_directive", body: {text}}` — a line inside the group that does not parse, recorded whole. An ignored group need only be lexically valid (§22.6), and whether a group is ignored is the expansion stage's to decide; the text is still lexed, so an unclosed string or comment refuses the file. At the top level the same line is refused. The conditional keywords are never recorded so. |

**The condition** (`macro`) takes one of three shapes:

| shape | when | example |
|---|---|---|
| `[[trivia], "NAME"]` — the identifier envelope | a `text_macro_identifier`, every profile | `` `ifdef DEBUG `` → `[[" "], "DEBUG"]` |
| `` {kind: "macro_usage", name: [[trivia], "`M"], arguments} `` | a macro usage, every profile (a defined macro *"can be used anywhere"*) | `` `ifdef `M `` |
| `{kind: "expression", expression: <node>}` | `( ifdef_macro_expression )`, `sv_2023` / `sv_2023_tolerant` only | `` `ifdef (A && !B) `` |

The expression `node` is `{kind: "identifier", name}`, `{kind: "not", operand}`, `{kind: "binary", operator, left,
right}` (`operator` one of `"&&"`, `"||"`, `"->"`, `"<->"`; `!` binds tightest, then `&&`, `||`, and `->` / `<->`
lowest, right-associative), `{kind: "group", expression}` for a parenthesized sub-expression, or a `macro_usage`.

**Consumer walk.** Dispatch a `pp_item` on `kind == "conditional"`; read `body.if_branch.keyword.kind` and its
`macro`, dispatching the condition on its shape (an array is the identifier envelope, an object has a `kind`); recurse
`items`, handling `malformed_directive` as an opaque line; iterate `body.elsif_branches`; then `body.else_branch` (`[]`
when absent). The tree records every group — this parser does not evaluate conditions.

## Family: the 1800-2023 condition expression (`sv_2023` / `sv_2023_tolerant`)

Through schema `6` the `` `elsif `` condition was `condition_expr`, a flat 12-kind atom list running to the line end;
it is GONE in schema `7`. The condition is now one of the three shapes in the table above; the third, an
`( ifdef_macro_expression )`, derives only under the 1800-2023 profiles:

```ebnf
ifdef_paren_expression := lparen ifdef_implication rparen -> {kind: "expression", expression: $2}
ifdef_implication := ifdef_or implication_op ifdef_implication -> {kind: "binary", operator: $2, left: $1, right: $3}
                   | ifdef_or -> $1
ifdef_or  := ifdef_and logical_or ifdef_or   -> {kind: "binary", operator: "||", left: $1, right: $3} | ifdef_and -> $1
ifdef_and := ifdef_unary logical_and ifdef_and -> {kind: "binary", operator: "&&", left: $1, right: $3} | ifdef_unary -> $1
ifdef_unary := bang ifdef_unary -> {kind: "not", operand: $2} | ifdef_primary -> $1
ifdef_primary := lparen ifdef_implication rparen -> {kind: "group", expression: $2}
               | identifier -> {kind: "identifier", name: $1}
               | p_macro_usage -> $1
```

`` `ifdef (A && !B || C -> D) `` (the release probe, `--profile sv_2023`) gives
`{kind: "binary", operator: "->", left: {kind: "binary", operator: "||", left: {kind: "binary", operator: "&&", …}, …},
right: {kind: "identifier", name: [[" "], "D"]}}` — `!` binds tightest, then `&&`, `||`, and `->` / `<->` lowest.

## Family: macro formals + default values (7-kind atom + the balanced group)

```ebnf
macro_formals := formals_open macro_formal (formal_comma macro_formal)* formal_rparen
              -> [$2, $3::2*]
formals_open := /\(/
macro_formal := formal_name formal_default?
             -> {name: $1, default: $2}
formal_default := formal_assign macro_default_value?   # @profiles: the IEEE 1800 editions
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

⭐ **Release `1.0.9`** (ledger `SVPP-0013`): a `(` right after the name ALWAYS opens the list — a list that does not
parse makes the define malformed, never object-like (through `1.0.8` it fell back to an object-like define whose text
began `(`: UVM-core's `` `define uvm_declare_type_alias(TYPE,NAME,SFX=) `` among 15 corpus files). The list's layout is
white space ONLY — a comment between formals is refused (§22.5.1: *"separated by commas and optionally white space"*) —
and a newline is white space under `sv_2017` / `verilog_2005`, not under `sv_2023` (§22.2's one-line rule). An EMPTY
default (`b=)`) is a default with no atoms; a default of any kind is IEEE 1800 only.

## Family: macro body (12-kind fragment)

```ebnf
macro_body := macro_body_fragment+
           -> {fragments: $1}
macro_body_fragment := macro_line_continuation -> {kind: "line_continuation"}
                     | macro_token_paste -> {kind: "token_paste"}
                     | macro_stringize   -> {kind: "stringize"}
                     | macro_escaped_quote -> {kind: "escaped_quote"}
                     | macro_directive_word -> {kind: "directive", body: $1}
                     | macro_reference   -> {kind: "macro_reference", body: $1}
                     | macro_body_text   -> {kind: "text",   body: $1}
                     | macro_lparen      -> {kind: "lparen"}
                     | macro_rparen      -> {kind: "rparen"}
                     | macro_comma       -> {kind: "comma"}
                     | macro_question    -> {kind: "question"}
                     | macro_colon       -> {kind: "colon"}
```

| Rule | Shape | Notes |
|---|---|---|
| `macro_body` | `{fragments}` | `fragments` is the recursive-envelope `macro_body_fragment+` array (the macro replacement text, tokenized). |
| `macro_body_fragment` (12 kinds) | `{kind, body}` for `"macro_reference"` / `"text"`; bare `{kind}` for `"line_continuation"`, `"token_paste"`, `"stringize"`, `"escaped_quote"`, `"lparen"`, `"rparen"`, `"comma"`, `"question"`, `"colon"`. The macro-body atom set adds `"comma"`, `"line_continuation"` and `"escaped_quote"` (vs the 8-kind `macro_default_atom`) but, like the default-atom set, has no `"logical_or"` / `"logical_and"` / `"bang"`. |

The `"token_paste"` fragment is the SystemVerilog `` `` `` paste operator; `"stringize"` is the `` `" `` stringize operator; `"escaped_quote"` is `` `\`" `` (§22.5.1: *"The `` `define `` macro text can also include `` `" ``, `` `\`" ``, and ``` `` ```"*; `1.0.8`). Each `"text"` fragment carries the un-annotated `macro_body_text` envelope (a run of literal replacement characters).

⭐ **Three changes in `1.0.8` a consumer of the fragments sees** (schema `5` → `6`):

- **`"line_continuation"`** — a backslash immediately before the newline continues the macro text on the next line (§22.5.1). Through `1.0.7` the body ENDED at the first newline: `` `define M(a) a + \ `` + `  b` gave the text `"a + \"` and a separate `non_directive_line` item for `  b` (ledger `SVPP-0006`). Now the fragments run `text "a + "`, `line_continuation`, `text "b"`, and no stray item follows. A backslash followed by white space is not a continuation.
- **A plain string literal is ONE text atom** — §22.5.1 *"Macro substitution and argument substitution shall not occur within string literals"*: `$display("got `%b`, (%0d)", v)` keeps `"got `%b`, (%0d)"` whole, where the run used to stop at the backtick (refusing the define) or split at `(`, `,`, `:`. `` `define BACKTICK "`" `` no longer reads its `` `" `` as a stringize marker.
- **A `//` comment is ONE text atom**, up to — not including — a continuation backslash (IEEE 1800-2023: *"If a one-line comment is followed by a backslash and newline character, the comment ends before the backslash and the macro text continues on the next line"*).

⭐ **Release `1.0.9`** (schema `6` → `7`): a DIRECTIVE inside macro text — `` `define X `ifdef A 1 `endif `` — is a
`` {kind: "directive", body: [[trivia], "`ifdef"]} `` fragment, processed where the macro is used (1800-2023: *"Any compiler
directives appearing within the macro text shall be ignored until the macro is used"*); through `1.0.8` it was a
`macro_reference` named `` `ifndef `` (23 corpus files' trees move, OpenTitan's `prim_assert_standard_macros.svh` among
them). The fragments' leading layout is macro text's own: a block comment there crosses a line only where each newline
is backslash-preceded under `sv_2017` / `verilog_2005`, anywhere under `sv_2023` (BlackParrot's
`bp_common_core_if.svh` continues its comments so). A define ends at its line end, never mid-line, and the
punctuation fragments are regex terminals — through `1.0.8` their default layout skip let a body run on into a next
line opening with `(` (`SVPP-0014`). `` `" ``, `` `\`" `` and ``` `` ``` are IEEE 1800 only.

## Family: passthrough text (release `1.0.9`)

```ebnf
pp_non_directive_line := !directive_keyword non_directive_text text_run_end -> {text: $2}
non_directive_text := inline_trivia text_run
text_run := text_lexeme+ -> $text
text_lexeme := text_block_comment | text_triple_string | text_line_comment | text_string
             | text_escaped_identifier | text_macro_usage | text_chars | text_slash
text_run_end := newline | &mid_line_directive | end_of_input
pp_blank_line := newline -> {kind: "blank"}
```

| Rule | Shape | Notes |
|---|---|---|
| `pp_non_directive_line` | `{text}` | Text that is not a directive, LEXED: a block comment (it may span lines), a line comment, a string (one line, or continued by a backslash-newline, §5.9), a triple-quoted string (`sv_2023`), an escaped identifier (A.9.3; the empty one included), a text macro usage, other characters. A directive word inside a comment or string is text; an unclosed string or comment refuses the file. The item ends at a line end, at the end of input, or before a directive that may stand mid-line — so `` assign a = `ifdef X 1 `else 0 `endif ; `` is a `non_directive_line` (`"assign a = "`), a `conditional`, and another `non_directive_line` (`" ;"`). `` `include `` and `` `line `` own their line. `text` is still ONE string (`$text`), as the `1.0.8` line was. |
| `pp_blank_line` | `{kind: "blank"}` | A blank line; `{kind: "blank_line"}` at the `pp_item` dispatch. |
| `pp_group_item` → `malformed_directive` | `{kind: "malformed_directive", body: {text}}` | Inside a conditional group only: a line that does not parse, recorded whole (§22.6; see the conditional family above). |

## Atom-set summary

| Rule | Branches |
|---|---|
| `macro_body_fragment` | 12 — `line_continuation`, `token_paste`, `stringize`, `escaped_quote`, `directive`, `macro_reference`, `text`, `lparen`, `rparen`, `comma`, `question`, `colon` |
| `macro_default_atom` | 6 kinds + the balanced `paren_group` |

(`condition_atom`, the third atom family through schema `6`, is gone: the `` `elsif `` condition is an identifier, a
macro usage or a 2023 expression.)

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.9` is **112 return annotations across 60 distinct rules** — **95 `annotation_type: "return_object"` + 14 `return_scalar` + 3 `return_array`** (the arrays are `macro_formals`, `pragma_expression_list` since `1.0.8`, and `macro_simple_name` since `1.0.9` — its `[$1, $3]` rebuilds the name envelope around the lookahead that refuses a directive's name; the 14 scalars are `$text` / passthrough results: `text_run`, `group_lexical_line`, `macro_text_run_line`, `macro_text_run_2023`, `macro_usage_arguments`, `pragma_value_list`, `macro_default_group_atom`, the five `ifdef_*` levels and `implication_op`'s two). The history: 64 / 27 at SVPP-Slice-1, 66 / 28 after `1.0.2`'s `pp_if_keyword`, 67 / 29 at `1.0.5`, 68 / 31 at `1.0.7`, **+32 / +15 at `1.0.8`** — the eight new `pp_item` branches, the eight directive rules and their `pull_drive` / `pragma_*` value grammar, `include_macro_usage`, and the two new `macro_body_fragment` kinds — and **+12 / +14 at `1.0.9`** (19 rules added — the group fallback, the text run, the 2023 expression, the parameter macro usages, the escaped-formals define, `macro_simple_name` — and 5 removed: `condition_expr` / `condition_atom` / `condition_text`, `pull_drive`, `time_literal`). This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/systemverilog_preprocessor_return_annotations.json` — the live return-annotation inventory (`version: 1`, `grammar: "systemverilog_preprocessor"`, `annotation_count: 112`).
- `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` — the embedded inventory used by the AST shape-contract regression lock (`declared_annotation_inventory.annotations`, 112 entries; identical tuples — the contract-embedded copy omits only the cosmetic `raw_text` field), beside thirteen samples whose JSON-pointer claims pin the trees `1.0.8` changed or added.

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after SVPP-Slice-1 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together.
