# Glossary

Terms used throughout this book. Where a term has a normative definition, the integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` is authoritative; this glossary paraphrases it for quick lookup.

## AST envelope

The JSON tree returned by the sv_preprocessor AST-dump entry points. It is the `root` field of the `AstDumpPayload` carried by `NamedGrammarAstDumpOutcome` / `GrammarAstDumpOutcome`. Every parse roots at the single typed object `{type: "systemverilog_preprocessor_file", items: [ <pp_item> … ]}`. See [AST Envelope Structure](ast-envelope.md).

## AST shape contract manifest

The file `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (`version: 1`, `grammar: "systemverilog_preprocessor"`). It records the per-rule expected JSON shape for each sample — thirteen since `1.0.8`: the six earlier ones (`single_define`, `conditional`, `macro_with_formals`, `macro_body_comment_backtick`, `condition_comment_special_char`, `macro_default_on_last_formal`) and seven whose JSON-pointer claims pin the trees `1.0.8` added or corrected (`clause22_directives`, `pragma_expression_forms`, `macro_usage_line`, `define_continuation`, `define_space_before_parenthesis_is_object_like`, `macro_text_escaped_quote_and_string`, `include_macro_path`) — and embeds the declared-annotation inventory: **100 annotation entries across 46 distinct rules** as of contract `1.0.8` / schema `6`. It is the regression lock `ast_shape_contract_gate` runs.

## Conditional-compilation tree

The recursive `` `ifdef`` / `` `ifndef`` / `` `elsif`` / `` `else`` / `` `endif`` chain, modelled by typed rules: `pp_conditional` (`{if_branch, elsif_branches, else_branch}` — the terminating `pp_endif` is matched but not surfaced), `pp_if_branch` (`{keyword, macro, tail, items}`), `pp_elsif_branch` (`{condition, items}`), `pp_else_branch` (`{tail, items}`), `pp_endif` (`{tail}`), plus the 2-branch `pp_if_keyword` polarity dispatch rule (`{kind: "ifdef"}` / `{kind: "ifndef"}`) the `1.0.2` `SVPP-0001` fix added. Each branch's `items` is itself a `pp_item*` array, so a `pp_conditional` can nest inside any branch. `pp_if_branch.keyword` is now the typed `pp_if_keyword` polarity object `{kind: "ifdef"|"ifndef"}` — read the polarity from `if_branch.keyword.kind` (was the malformed `<invalid_sequence_access>` object at `1.0.1` / schema `1`, the `SVPP-0001` defect; see below). See [Top-Level Rules](rules-top-level.md) and the [Conditional Compilation](examples-conditional.md) worked example.

## Declared-annotation inventory

The machine-checkable enumeration of every typed-shape annotation the sv_preprocessor grammar emits: `generated/systemverilog_preprocessor_return_annotations.json` (`version: 1`, `grammar: "systemverilog_preprocessor"`, `annotation_count: 100`). It is the live source of truth for the typed surface and is mirrored content-for-content by the embedded inventory in `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` (100 entries). **94** entries are `annotation_type: "return_object"`, **4** `return_scalar` (`condition_text -> $text`, `macro_default_group_atom -> $1`, `macro_usage_arguments -> $text`, `pragma_value_list -> $2`) and **2** `return_array` (`macro_formals`, `pragma_expression_list`), over 46 distinct rules.

## {first, rest} list convention (macro_formals — historical, removed in 1.0.3)

The raw separated-list carrier `{first: <head element>, rest: <raw [[separator, element], …] iteration>}`. In the sv_preprocessor grammar this was used in **exactly one** rule — `macro_formals` (`lparen macro_formal (comma macro_formal)* rparen -> {first: $2, rest: $3}`) — at ≤ release `1.0.2` / schema `2`. It was a Category-A raw-envelope misuse: the `comma` separator carried no payload, yet a consumer had to walk past `rest[i][1]` to reach each `macro_formal`. **Corrected in `1.0.3` / schema `3`** (POST-SV-AUDIT.2.1, `PGEN-POST-SV-AUDIT-0002`): `macro_formals` was flattened to the clean `[$2, $3::2*]` extraction-spread, so `pp_define.formals` is now a **flat `macro_formal[]` array** (the `return_array` glossary term), mirroring the SystemVerilog grammar's slice-58 list flatten. The other token/atom lists (`condition_expr.atoms`, `macro_default_value.atoms`, `macro_body.fragments`) were always plain `x+` arrays, never `{first, rest}`. This term is retained only for consumers reading code vendored against ≤ `1.0.2`; the current shape is documented under [Walking the AST](walking-the-ast.md#iterating-the-macro_formals-list) and the schema-`3` row of [Changelog Index](changelog-index.md) / [Schema Versioning](schema-versioning.md).

## `<invalid_sequence_access>` (SVPP-0001 — resolved defect)

The literal sentinel string PGEN's emit-time machinery substitutes when a bare positional `$N` reference resolves to an **inline alternation group**. In sv_preprocessor this affected exactly one field — `pp_if_branch.keyword` — and is **resolved as of parser release `1.0.2` / schema `2`** (`SVPP-0001`). *Historical (release `1.0.1`, schema `1`):* for `` `ifdef`` / `` `ifndef`` conditional input, `items[].body.if_branch.keyword` was a malformed nested object containing three `"<invalid_sequence_access>"` strings instead of the keyword token, because `pp_if_branch := (kw_ifdef | kw_ifndef) macro_name … -> {keyword: $1, …}` bound `$1` to the inline `(kw_ifdef | kw_ifndef)` group — the same emit-time defect class fixed for `rtl_const_expr` in RTL-CE-Slice-2 and tracked for `rtl_frontend` / `vhdl` `binop_chain`. The `` `define`` / non-conditional surface was always unaffected. **Fix:** the inline alternation is lifted into the named rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef -> {kind: "ifndef"}` (the proven RTL-CE-Slice-2 / `systemverilog.ebnf` idiom); `pp_if_branch`'s bare `keyword: $1` now binds the clean named rule, so `if_branch.keyword` is the typed polarity object `{kind: "ifdef"}` / `{kind: "ifndef"}` with no `<invalid_sequence_access>` anywhere. **Current consumer guidance:** read the conditional polarity from `if_branch.keyword.kind` and the guard macro from `if_branch.macro`. Status `Released`; the schema-`1`→`2` transition is shown with the pre-fix history kept honestly in [Conditional Compilation](examples-conditional.md); recorded in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Resolved Defects — `SVPP-0001`" and `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`).

## macro_body_fragment (12 kinds)

The `kind`-tagged dispatch for the tokenized `` `define`` replacement text (`macro_body := macro_body_fragment+ -> {fragments: $1}`). The **12** kinds are `line_continuation`, `token_paste`, `stringize`, `escaped_quote`, `directive` (`{kind, body}`), `macro_reference` (`{kind, body}`), `text` (`{kind, body}`), `lparen`, `rparen`, `comma`, `question`, `colon` (9 through `1.0.7`; `line_continuation` and `escaped_quote` since `1.0.8`; `directive` since `1.0.9` — a directive inside macro text, through `1.0.8` a `macro_reference`). `"macro_reference"` and `"text"` carry a `body`; the rest are bare `{kind}` atoms. `"token_paste"` is the SystemVerilog ``` `` ``` operator, `"stringize"` the `` `" `` operator, `"escaped_quote"` the `` `\`" `` escape. A `"text"` fragment keeps a plain string literal and a `//` comment whole (since `1.0.8`). See [Top-Level Rules](rules-top-level.md#family-macro-body-12-kind-fragment).

## macro_default_atom (7 kinds + the balanced group)

The `kind`-tagged dispatch for a macro formal's default value (`macro_default_value := macro_default_atom+ -> {atoms: $1}`). Seven kinds — `token_paste`, `stringize`, `macro_reference` (`{kind, body}`), `text` (`{kind, body}`), `question`, `colon` — plus the balanced `{kind: "paren_group", atoms}` (since `1.0.7`, `SVPP-0004`), inside which, and only there, a comma surfaces as `{kind: "comma"}`. A `"text"` atom keeps a string literal whole since `1.0.8`, so a quoted comma (`a="x,y"`) no longer ends the default.

## condition_atom (12 kinds — removed in 1.0.9)

Through schema `6`, the `` `elsif `` condition was a flat 12-kind token list (`condition_expr := condition_atom+ -> {atoms: $1}`) running to the line end — which swallowed code written after the condition (ledger `SVPP-0012`). REMOVED in release `1.0.9` (schema `7`): an `` `elsif `` takes the `` `ifdef `` condition, `{macro, tail, items}` — an identifier, a macro usage, or under `sv_2023` an `( ifdef_macro_expression )` tree. See [Top-Level Rules](rules-top-level.md).

## parseability_probe

The CLI wrapper around `pgen::embedding_api` used for terminal-side verification, AST inspection, and bug-report reproducers. Sub-commands include `--parse`, `--parse-dump-ast`, and `--parse-dump-ast-pretty`. For sv_preprocessor the parser is on-demand-only, so the probe must be built with the generated backend before use (see [Build Recipe](build-recipe.md)). The full flag set, exit codes, and registered grammars are in the [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md).

## Clause-22 directive

A compiler directive of IEEE 1800 clause 22 — the twenty keywords §22.1 lists, minus `` `__FILE__ `` / `` `__LINE__ ``, which §22.13 makes predefined text MACROS. Since `1.0.8` every one has a production (`SV-CORPUS-GRAD.13c.2v.1`). The grammar's `directive_keyword` rule lists them, and a negative lookahead over it is what keeps a malformed directive from re-reading as a macro-usage line.

## Macro-usage line

A source line that OPENS with a text macro usage — `` `uvm_object_utils(my_comp) ``, `` `ASSERT(name, prop) ``, `` `__LINE__ `` (IEEE 1800 §22.5.1 Syntax 22-3). It is a `non_directive_line`: this parser records directive structure and passes source text through; expanding the usage is the expansion stage's job. Refused through `1.0.7` — 4,182 of the 16,336 SystemVerilog corpus files stopped at one (`SV-CORPUS-GRAD.13c.2v.2`).

## Line continuation

A backslash immediately before the newline inside `` `define `` macro text: the text continues on the next line (§22.5.1). It surfaces as a bare `{kind: "line_continuation"}` fragment between the two lines' fragments. Through `1.0.7` the body ended at the first newline (`SVPP-0006`). A backslash followed by white space is NOT a continuation.

## Corpus census (this parser over the SystemVerilog corpus)

`docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py`: the GENERATED parser inside the release `parseability_probe`, run over every file the SystemVerilog lanes read, each rejection classed by the backtick word that opens the line where the parse stopped and joined to a written disposition (`residual_dispositions.tsv`); `--before BIN` adds transitions and AST identity against another binary; `--bank` runs the clause-22 reproducer bank. See [Status and Distance](status-and-distance.md#the-systemverilog-corpus-read-by-this-parser-2026-10-17).

## Over-acceptance

A file the parser ACCEPTS that its requirement refuses — the opposite of a rejection, and invisible to a census that reads only
rejections, because an accepted file never reaches the disposition table. Two were found in release `1.0.8` this way (ledger
`SVPP-0010`, `SVPP-0011`; see [Status and Distance](status-and-distance.md#two-wrong-verdicts-the-census-could-not-see-found-2026-10-18)).
Measuring it needs an answer key that says which files must be refused: the next release joins the SystemVerilog answer key's
`must_reject` and `deferred:svpp_owned` rows to this parser's verdict.

## Parser release version

The parser library's release identity, currently `1.0.8`. Bumped on every functional change to the parser, including bug fixes, performance work, and grammar changes. It moves independently of the schema version: a release can carry the same schema version as the previous one (no shape change) or a bumped one (shape changed) — the `1.0.2` `SVPP-0001` fix carries schema `2`, the `1.0.3` POST-SV-AUDIT `macro_formals` Category-A fix carries schema `3`, `1.0.5` (`condition_text -> $text`) carries schema `4`, the `1.0.6` `SVPP-0003` acceptance fix keeps schema `4`, and the `1.0.7` `SVPP-0004` macro-default correction carries schema `5`. Recorded in `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". See [Schema Versioning](schema-versioning.md).

## pp_item dispatch

The primary top-level dispatcher of the sv_preprocessor AST. `pp_item` is an **18-branch** `kind`-tagged shape — `"define"`, `"undef"`, `"include"`, `"timescale"`, `"default_nettype"`, `"celldefine"`, `"endcelldefine"`, `"resetall"`, `"undefineall"`, `"unconnected_drive"`, `"nounconnected_drive"`, `"pragma"`, `"line"`, `"begin_keywords"`, `"end_keywords"`, `"conditional"`, `"non_directive_line"`, `"blank_line"` (10 through `1.0.7`) — each carrying a `body` except the bodyless `"celldefine"` / `"endcelldefine"` / `"blank_line"`. Every parse roots at `{type: "systemverilog_preprocessor_file", items}`. Dispatch on `kind`; the branch indices shifted in `1.0.8`. See [Top-Level Rules](rules-top-level.md#pp_item-dispatch).

## Profile

A named EDITION reading of the grammar (`@profiles` gates). Since release `1.0.9` the family has the SystemVerilog
family's four: `sv_2017` (IEEE 1800-2017, **the default** — `@default_profile`), `sv_2023` (IEEE 1800-2023),
`sv_2023_tolerant` (as `sv_2023` for every directive) and `verilog_2005` (IEEE 1364-2005), with the aliases `2017`,
`2023`, `1364-2005`, `ieee1800-2017`, … Pass one with `parseability_probe --parse systemverilog_preprocessor FILE
--profile sv_2023`, or `set_grammar_profile(Some("sv_2023"))` on the generated parser; `None` restores the default. The
entry rule is `systemverilog_preprocessor_file` under every profile. See [Editions and Profiles](profiles.md). (Through
`1.0.8` the family had one profile, `default`, with no edition axis.)

## Edition contrast

A form two editions read differently — `` `ifdef (A) `` derives under `sv_2023` and is refused under `sv_2017`; a
`` `timescale `` written over two lines is the reverse. The census marks such a refusal `edition_contrast:<profile>`
(the profile under which the file derives). Every contrast is a bank case under each profile it splits.

## Line discipline

Whether a directive's tokens may be separated by NEWLINES. IEEE 1800-2023 §22.2: *"Unless specified below as supporting
multiple lines, compiler directives shall be all on one line"* — only the macro text and a conditional's block of text
are so specified. IEEE 1364-2005 and 1800-2017 have no such sentence, so under `verilog_2005` / `sv_2017` a directive
may be written over several lines (iverilog's `br_gh782b.v` writes a `` `timescale `` one token per line), a formal
list included; `` `include `` and `` `line `` own their line in every edition. Under `sv_2023` a block comment that
crosses a line INSIDE a directive breaks the rule too (release `1.0.9`): the comment carries the directive onto a second
line. Macro text keeps 1800-2023's own licence for multi-line comments.

## Malformed directive (group item)

`{kind: "malformed_directive", body: {text}}` — a line inside a CONDITIONAL GROUP that does not parse, recorded whole
instead of refused (release `1.0.9`). §22.6: an ignored group need only be lexically valid, and whether a group is
ignored is decided by the expansion stage, which has the compilation's symbols and includes. The line is still lexed
(an unclosed string or comment refuses the file); at the top level the same line is refused; the conditional keywords
are never recorded so. See [Groups, Comments and Lines That Do Not Parse](examples-groups.md).

## Text run (lexeme)

The passthrough text of a line, read as LEXEMES since `1.0.9`: block comments (which may span lines), line comments,
strings (one line, or backslash-continued), triple-quoted strings (`sv_2023`), escaped identifiers, macro usages and
other characters. A directive word inside a comment or a string is text, and a directive word outside them ends the
run, so a directive may begin mid-line. The tree keeps the run as ONE string (`text: [[trivia], "…"]`).

## Actual arguments (of a macro usage in a directive)

The `( … )` after a macro usage that stands in a directive's parameter (`` `include `M(f) ``, `` `timescale `T / 1ps ``),
kept as ONE string in `arguments`. Since release `1.0.9` it is LEXED per §22.5.1: a string, a block comment and an
escaped identifier are single atoms, and a parenthesis inside `[ ]` or `{ }` is a character, so only a `)` outside
all of them ends the list (`` `include `M(")") ``). Ledger `SVPP-0015`.

## Certificate (proof / witness / UNKNOWN)

Every grammar rule is either PROVEN unreachable or WITNESSED by a generated sample whose parse exercises it; a rule
with neither is UNKNOWN. The grammar is *fully certified* when nothing is UNKNOWN and no proof fails re-verification; a
generated sample the parser refuses is reported beside it (`sample_parse_failures`) and read, never discounted. The
published bar is seed 0, 40 samples, the base configuration (`docs/book/src/grammar-certification-status.md`).

## Sample parse failure

A sample the certificate's generator wrote from the grammar that the generated parser then refuses. It is one of two
things, never discounted: a LEXICAL BOUNDARY the grammar never declared for generation (the parser is right; the
generator concatenated two tokens that re-read differently), or a parser DEFECT (the text is valid and refused).
Release `1.0.9`'s first build had 101 over 32 seeds; reading each one found three parser defects no corpus file
holds. See *Follow restriction* below.

## Follow restriction (`[>! … ]`)

A grammar annotation on a rule: its match must not be followed by the listed pattern. The parser ignores it (the
rule's own regex already holds the boundary); the stimuli generator appends the separator the pattern cannot start.
This grammar declares with it that a `/` before `/` or `*` is a comment opener, that an escaped identifier ends at white
space, that a `//` comment ends its line, and that an object-like macro's name is not followed by `(`.

## End-of-input anchor

A terminal that matches only at the end of the text (`end_of_input := /\z/`), used where a line construct may end the
file without a newline (a define's last line, a passthrough line, a `` `line ``). The generator renders it as nothing,
so it draws an alternative holding one only where nothing mandatory follows — never inside a conditional group, whose
closing keyword follows.

## Error-test half (of the census)

The census's over-acceptance reading: it selects the corpus's preprocessor-themed suite error tests by each suite's
naming convention and requires every one the parser ACCEPTS to carry a written reason (`error_test_dispositions.tsv` —
the error is the expansion stage's or SystemVerilog's, not the directive structure's). Headline
`SVPP-CORPUS-CENSUS-ERROR-TESTS: … accepted=A unadjudicated_accepts=U`. See [Over-acceptance](#over-acceptance).

## Recursive envelope

The default JSON shape produced by un-annotated rules — a recursive composition of arrays (for sequences and the `x*` / `x+` quantified iterations; note `macro_formals` is now the clean typed `return_array` flat list, not a recursive `{first, rest}` envelope, as of `1.0.3` / schema `3`), strings (for terminal and regex leaves), and matched-branch passthroughs (for alternations). An un-matched optional is the empty array `[]`, never `null` (verified empirically — e.g. `` `define MAX`` captures `{name: …, formals: [], body: []}`). In sv_preprocessor the recursive envelope is what you reach when you descend below the typed surface: the identifier / macro-name tokens, the literal-text runs (`macro_body_text`, `macro_default_text`, `non_directive_text`, `directive_tail`, `directive_comment_tail` — but **not** `condition_text`, which is a flat `$text` string since `1.0.5`/schema `4`), `unsigned_number`, `time_unit`, `macro_reference`, and the keyword / punctuation token rules. See [AST Envelope Structure](ast-envelope.md) and [The Json Carrier](json-carrier.md).

## Return annotation

A `-> ...` clause appended to a grammar rule definition in `grammars/systemverilog_preprocessor.ebnf` that overrides the default recursive-envelope shape with a typed JSON value. Example: `systemverilog_preprocessor_file := pp_item* -> {type: "systemverilog_preprocessor_file", items: $1}`. The annotation language (`$N` positional references, `{field: value}` object literals, `[...]` array literals, string literals) is specified in `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. The sv_preprocessor surface uses three annotation kinds — `return_object` (an object literal, **65** of the 68 annotations), `return_array` (an array literal, the single `macro_formals` annotation, the `1.0.3` POST-SV-AUDIT Category-A `[$2, $3::2*]` extraction-spread), and `return_scalar` (**2**: `condition_text -> $text` since `1.0.5`/schema `4`, and `macro_default_group_atom` since the `1.0.7` `SVPP-0004` fix); there is no `binop_chain`-style expression carrier (the grammar is directive / line-oriented, not expression-precedence-oriented).

## Schema version

Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to (a new annotation on a previously-unannotated rule, a restructured annotation, a user-visible grammar-shape change). Pure performance work and internal codegen reorganization do **not** bump it. The AST-dump schema version is currently `5` — it was bumped `1 → 2` by the `1.0.2` `SVPP-0001` correctness fix (the `pp_if_branch.keyword` shape changed from the malformed `<invalid_sequence_access>` object to the typed `{kind: "ifdef"|"ifndef"}` `pp_if_keyword` polarity discriminator), `2 → 3` by the `1.0.3` POST-SV-AUDIT Category-A `macro_formals` correction (`pp_define.formals` changed from the raw `{first, rest}` iteration envelope to the clean flat `macro_formal[]` list — a deliberate audit-driven shape correction, `PGEN-POST-SV-AUDIT-0002`, **not** a released-parser bug), `3 → 4` at `1.0.5` (`condition_text -> $text` — a `condition_atom` "text" atom's `body` became a flat string), and `4 → 5` by the `1.0.7` `SVPP-0004` macro-default balanced-parentheses correction (default-on-last-formal now yields structured formals; paren-bearing defaults yield one `paren_group` atom). Note it is **not** a field of `AstDumpPayload` (that struct has only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`); consumers **pin** the schema version they built against from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" and re-check the contract's "Schema Versioning" table when bumping PGEN. That table uses the integer `5` / `4` / `3` / `2` rows plus `1.0.0` / `0.1.0` milestone labels for the typing-campaign timeline (schema `5` @ release `1.0.7`; schema `4` @ releases `1.0.5`–`1.0.6`; schema `3` @ releases `1.0.3`–`1.0.4`; schema `2` @ release `1.0.2`; schema `1.0.0` @ release `1.0.1`; schema `0.1.0` @ release `1.0.0`); use the milestone labels when reading the changelog. See [Schema Versioning](schema-versioning.md).

## Typed shape

The JSON value produced by an annotated rule. In sv_preprocessor it takes three sub-forms: a **root object** carrying `type` (only `systemverilog_preprocessor_file`); a **`kind`-tagged dispatch object** (`pp_item`, `include_path`, `nettype_value`, and the three atom/fragment dispatchers); and a **named-field object** for single-sequence rules (`pp_define`, `pp_undef`, `pp_include`, `pp_timescale`, `pp_default_nettype`, the conditional-tree rules, `macro_formal`, `time_literal`, …). There is **no** `binop_chain` / expression carrier. Consumers dispatch on `obj["type"]` at the root and `obj["kind"]` for variants. See [The Json Carrier](json-carrier.md) and [Top-Level Rules](rules-top-level.md).

## Un-annotated leaf (rule of thumb)

A value reached through an un-annotated rule surfaces as that rule's **recursive envelope**, not a bare JSON string. The text is nested inside the envelope, reached by walking to the terminal — for an identifier or text run this is typically `[ <trivia-prefix>, "<text>" ]`, with the text at index `[1]`. Do **not** assume any identifier-valued or text-valued field (`pp_define.name`, `pp_if_branch.macro`, every `"text"` atom's `body`, `time_literal.value` / `.unit`, the `*_comment` / `tail` fields) is a scalar string: in the [single-define worked example](examples-single-define.md), `pp_define.name` is `[ [ " " ], "FOO" ]`, not the string `"FOO"`. The robust consumer rule is to walk to the terminal string rather than indexing a fixed depth. See [The Json Carrier](json-carrier.md) and [Walking the AST](walking-the-ast.md).

## SVPP-Slice-1

The single comprehensive typing slice (landed 2026-05-14, parser release `1.0.1`, schema version `1`) that typed the entire `grammars/systemverilog_preprocessor.ebnf` directive surface at once — the `systemverilog_preprocessor_file` root; the 10-kind `pp_item` dispatch; the 7 per-directive shapes (`define` / `undef` / `include` / `timescale` / `default_nettype` / `celldefine` / `endcelldefine`); `include_path` / `nettype_value` / `time_literal`; the 5-node conditional-compilation tree; `condition_expr` / `condition_atom` (12 kinds); `macro_formals` / `macro_formal` / `macro_default_value` / `macro_default_atom` (8 kinds); `macro_body` / `macro_body_fragment` (9 kinds); and the passthrough lines — **64 return annotations across 27 distinct rules**, on top of the pre-typing baseline of one root annotation. Unlike the main SystemVerilog and regex parsers, whose return annotations were added rule-by-rule over a long per-slice campaign (each slice bumping the schema version), sv_preprocessor was typed in one pass. The slice shipped one annotation defect, `SVPP-0001` (above) — **since fixed in parser release `1.0.2` / schema `2`**, which lifted the inline `(kw_ifdef | kw_ifndef)` alternation into the named `pp_if_keyword` rule and took the surface to 66 annotations / 28 distinct rules. A subsequent POST-SV-AUDIT Category-A correction (parser release `1.0.3` / schema `3`, `PGEN-POST-SV-AUDIT-0002`) then corrected `macro_formals` from the raw `{first, rest}` envelope to the clean `[$2, $3::2*]` flat list (count unchanged at 66 / 28; only that rule's annotation form changed `return_object` → `return_array`). This batch-plus-two-fix history is why the sv_preprocessor [Changelog Index](changelog-index.md) and [Schema Versioning](schema-versioning.md) timeline are short by design. Subsequent shape-affecting slices, if any, each get their own contract row and changelog entry.

## Expander (the preprocessing service)

The second layer over this family's AST: it takes the directive AST plus a **context** (edition profile, predefined macros, include directories, the unit's earlier files, the declared policy options, and a **file provider** — `FileProvider`, *path → bytes* with `exists` and `canonicalize`, the disk by default or the consumer's own store, since `SVPP-EXPANSION.9.2`) and produces the **preprocessed output** (`text`, `source_map`, `events`, `diagnostics`, `included_files`). Today its logic is the hand-written expansion stage in `rust/src/families/systemverilog/preprocessor.rs`; `SVPP-EXPANSION.9.3` re-points it at this AST. Contract: `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md`; chapter: [Preprocessing as a Service](preprocessing-service.md).

## Consumption mode

One of the three first-class ways a downstream project uses this family together with the expander: **AST only** (the project expands itself or never expands), **PGEN expands** (the project compiles against the preprocessed output), **intercept** (the project edits the AST — injects a define, serves an include from its own store — and hands it to the expander). The first two are live; the third is contracted for `SVPP-EXPANSION.9.3`.

## File provider

The context member that answers *path → bytes* for an `` `include `` and for the unit's files. Its default reads the disk; a consumer's implementation serves memory, which is what makes the service usable in-process without a filesystem. Contracted for `SVPP-EXPANSION.9.2`.

## Preprocessed-output schema

The version axis of the expander's output, independent of this family's AST-dump schema: schema `1` is today's five-field output; schema `2` (`SVPP-EXPANSION.9.3`) adds the macro-expansion chain to every source-map entry. A change to one axis never silently changes the other.
