# docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `systemverilog_preprocessor` frontend/parsing stage.

## Contract Identity
- Contract version:
  - `1.0.9`
- Parser release version:
  - `1.0.9`
- systemverilog_preprocessor AST-dump schema version:
  - `7` (first release `1.0.9`; **per-edition profiles, lexed passthrough text, directives anywhere on a line, a conditional group's unparsable line recorded, and five corrected trees** — `SV-CORPUS-GRAD.13c.2v.8` – `.14`, ledger `SVPP-0010` – `SVPP-0014`; see [Editions and Profiles](#editions-and-profiles-release-109) and the schema-`7` row). Schema `6` (first release `1.0.8`) derived every IEEE 1800 clause-22 directive.
- Annotation count:
  - `112` (95 `return_object` + 14 `return_scalar` + 3 `return_array`; 60 distinct rules) — `1.0.9` adds the group fallback (`pp_group_item`, `pp_malformed_directive`, `group_lexical_line`), the lexed text run (`text_run`), the 2023 expression (`ifdef_paren_expression` … `implication_op`), the parameter macro usages (`p_macro_usage`, `line_macro_usage`), `p_time_literal` / `p_pull_drive`, the escaped-formals define, `macro_simple_name` and the macro-text runs, and removes `condition_expr` / `condition_atom` / `condition_text`, `pull_drive` and `time_literal`. `1.0.8` was `100` over 46 rules.
- Current grammar family label:
  - `systemverilog_preprocessor`
- Per-family mdBook:
  - `docs/systemverilog_preprocessor_parser_book/` (tracked HTML at `docs/systemverilog_preprocessor_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`
- Per-family ast-shape-contract manifest:
  - `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`

## Schema Versioning

The systemverilog_preprocessor parser carries two version axes:

1. **Parser release version** (`1.0.9`). Tracks the parser library's release identity.
2. **AST-dump schema version** (`7`). Tracks the AST output shape. Schema `7` (first release `1.0.9`) adds the per-edition profiles and corrects five trees (see the schema-`7` row). Schema `6` (first release `1.0.8`) derives every clause-22 directive and corrects three trees (see the schema-`6` row). Schema `5` (first release `1.0.7`) is the `SVPP-0004` macro-default balanced-parentheses correction. Schema `4` (first release `1.0.5`) annotated `condition_text -> $text`; release `1.0.6` kept schema `4`. Schema `3` spanned releases `1.0.3`–`1.0.4`.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 7 | 1.0.9 | **Per-edition profiles, lexed passthrough text, directives anywhere on a line, and five corrected trees (`SV-CORPUS-GRAD.13c.2v.8` – `.14`; ledger `SVPP-0010` – `SVPP-0014`).** Breaking for conditionals and macro text: an `` `elsif `` branch is `{macro, tail, items}` (was `{condition: {atoms}}`); a conditional's `tail` holds only a trailing comment; a group line that does not parse is a `malformed_directive` item; a condition may be a `macro_usage` or (under `sv_2023`) an `expression`; a directive in macro text is a `directive` fragment; a formal list never falls back to an object-like define; a `non_directive_line` may end before a mid-line directive. See [Editions and Profiles](#editions-and-profiles-release-109) and [Resolved Defects — SVPP-0010 … SVPP-0014](#resolved-defects--svpp-0010--svpp-0014-and-the-ordered-gaps-fixed-in-release-109-schema-6--7). |
| 6 | 1.0.8 | **Every IEEE 1800 clause-22 directive, a line that opens with a macro usage, and three corrected trees (`SV-CORPUS-GRAD.13c.2v.1` – `.7`; ledger `SVPP-0005` – `SVPP-0009`).** Found by the first corpus reading of this parser (`docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/`): the generated parser refused **4,477 of 16,336** SystemVerilog corpus files at `1.0.7`, 4,182 of them at a line that OPENS with a text macro usage. **Additive:** `pp_item` kinds `resetall`, `undefineall`, `unconnected_drive` (`{drive: {kind: "pull0"\|"pull1"}, comment}`), `nounconnected_drive`, `pragma` (`{name, expressions, comment}`, Syntax 22-8's `keyword_value` / `keyword` / `value` forms and nested lists), `line` (`{number, filename, level}`), `begin_keywords` (`{version, comment}`), `end_keywords`; `include_path` kind `macro` (`{name, arguments}`); a `non_directive_line` may open with a macro usage. **Breaking (the bump):** a backslash-continued define body is ONE define with `{kind: "line_continuation"}` fragments (was cut at its first line, the rest stray passthrough items — `SVPP-0006`); `` `define M (x) y `` is object-like (was function-like — `SVPP-0008`); a plain string literal and a `//` comment are each one `text` fragment of macro text, and `` `\`" `` is `{kind: "escaped_quote"}` (`SVPP-0009`); `` `include "a.sv" x `` is REFUSED (was accepted — `SVPP-0007`); the branch indices of `pp_item` and `macro_body_fragment` shift. Measured with `census.py --before` the `1.0.7` probe: rejections `4,477 → 64` (every one adjudicated), zero `pass → fail`, 11,790 of the 11,859 previously accepted trees byte-identical and the 69 that moved each explained (38 continuation, 31 string atom). Inventory `68 → 100`, distinct rules `31 → 46`. |
| 5 | 1.0.7 | **`SVPP-0004` macro-default balanced-parentheses correction (breaking for the buggy shapes, `GRAMMAR-WELLFORMED.H.9`; found by the `H.7.2` plannable-rule reach pass).** Per IEEE 1800 §22.5.1, macro default text may contain a comma or right parenthesis only *inside a balanced pair of parentheses* — but `macro_default_atom` modeled parens as bare `{kind:"lparen"}`/`{kind:"rparen"}` atoms, and the possessive `macro_default_atom+` swallowed the FORMALS' closing `)`. Consequence: a default on the **last** formal (`` `define M(a=x) y`` — legal SV; the LRM's own examples use it) made `macro_formals` fail, the `macro_formals?` optional backtracked to empty, and the directive mis-parsed to `formals: []` with the whole `(a=x) y` as macro-*body* fragments. Fixed by the LRM-faithful balanced group: `macro_default_paren_group := lparen macro_default_group_atom* rparen -> {kind: "paren_group", atoms: $2}` with `macro_default_group_atom := macro_default_atom -> $1 \| comma -> {kind: "comma"}` replacing the bare paren alternatives — an unbalanced `)` now correctly terminates the default, and commas are legal inside the group (and only there), both exactly per the LRM sentence. **Two consumer-visible shape changes (hence the schema bump):** (1) default-on-last-formal inputs now produce structured `formals` (e.g. `` `define M(a=x) y`` → `formals: [{name:"a", default:[…text "x"]}]`, body `"y"`) instead of the buggy empty-formals/body-text shape; (2) paren-bearing defaults that DID parse before (e.g. `a=(x),b`) now surface one `{kind:"paren_group", atoms:[…]}` atom instead of flat `lparen`/`rparen`/text atoms, and commas inside the group surface as `{kind:"comma"}` atoms (e.g. `a=(x,y)` → `paren_group{text "x", comma, text "y"}` — previously REJECTED-as-formals entirely). Annotation inventory `67 → 68` (adds `macro_default_paren_group` + the two `macro_default_group_atom` branches; removes the two bare paren branch annotations); distinct rules `29 → 31`. Verified by the decisive repro pair (`a=x` last-formal vs `a=x,b`), the svpp certificate-coverage sweep **`fully_certified=true` + `sample_parse_failures=0` at ALL 32 seeds 0–31** (this fix closed the 7 residual seeds), `sv_preprocessor_zero_plausible_gap_proof_gate` GREEN, and the new `macro_default_on_last_formal` sample in `systemverilog_preprocessor_v1.json`. Tracked `SVPP-0004` (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`). |
| 4 | 1.0.5 (also spans 1.0.6) | **`1.0.6` addendum — `SVPP-0003` `condition_text` comment-awareness fix (schema-neutral, non-breaking, `GRAMMAR-WELLFORMED.H.5.5`).** `condition_text := inline_trivia /[^`(),?:!\|&\r\n]+/` was **not comment-aware**: its content regex ate a comment's `/*` (both `/` and `*` are in its class) then halted at an operator char (`(`,`)`,`?`,`:`,`!`,`\|`,`&`) or backtick *inside* the `block_comment`, so `condition_expr` could not span the comment and a valid `` `ifdef``/`` `elsif`` condition containing such a comment (e.g. `` `elsif x /* a \|\| b */``) was wrongly rejected at `1.0.5` — the outer `pp_conditional` then could not close. Fixed in `1.0.6` by making the rule comment-aware with the proven `macro_body_text`/`macro_default_text` idiom (`/(?:\/\*([^*]\|\*+[^*\/])*\*+\/\|[^`(),?:!\|&\r\n])+/` — the `/*…*/` alt tried FIRST so a `/*` opener is consumed atomically; the unchanged char-branch still excludes the operators/backtick so they still split as their own atoms) AND **dropping the rule's now-redundant leading `inline_trivia`** (the comment-alt + char-class already admit comments/whitespace; keeping it would re-consume, as trivia, a comment the generator emits as content, breaking round-trip on a comment-only condition). **No schema bump**: `condition_text` stays `-> $text`, the annotation inventory is unchanged (67/29), and every input that parsed at `1.0.5` yields a byte-identical AST at `1.0.6` (the `$text` matched-SPAN is identical whether `inline_trivia` or the regex consumes a leading space — empirically `` `elsif abc`` → `body:" abc"` unchanged) — only previously-*erroring* comment-bearing conditions now succeed with the standard `{kind:"text", body}` shape (strictly more permissive). Tracked `SVPP-0003` (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`); locked by the `condition_comment_special_char` sample in `systemverilog_preprocessor_v1.json`. **`1.0.5` (original schema-`4` change) — `condition_text` annotated `-> $text` (consumer-visible, `GRAMMAR-WELLFORMED.H.5.1.3.2`).** `condition_text := inline_trivia /[^`(),?:!\|&\r\n]+/` previously had **no** return annotation, so it produced the raw `inline_trivia`+content envelope; a `condition_atom` text atom `{kind:"text", body:$1}` therefore surfaced `body` as that envelope (e.g. for `` `elsif abc`` → `body` was `[[" "], "abc"]`). It now declares `-> $text`, so `body` is the **flat matched-text string** (`" abc"`). This realizes the long-anticipated "annotate the literal-text runs" schema milestone for `condition_text` specifically — `condition_text` is a flexible **same-line** `condition_atom+` list element (its grammatical successors are excluded-delimiter atoms that self-delimit, or another text run that merges harmlessly), so it needs no line terminator; the line-oriented runs `directive_tail` / `non_directive_text` (which DO need a terminating newline) deliberately remain raw-envelope. **Why now:** the un-annotated rule made PGEN's stimuli generator inject a spurious trailing `\n` after a generated `condition_text` (its `\n`-excluding content class drove the word-boundary trailing guard to emit a newline), which prematurely ended a `` `elsif`` line and stranded a following `` `"`` stringize at a `pp_item` boundary — the lone svpp certificate-coverage `sample_parse_failures` residual. The `$text` annotation marks the rule one atomic lexical span (LEXICAL-ANNOTATIONS.6), which suppresses that trailing guard. Generator-faithfulness fix expressed declaratively at fix-hierarchy Level 1 (no engine change). **Result:** svpp `--report-certificate-coverage` `sample_parse_failures` `1 → 0` (deterministic, seeds 0 and 7); annotation count `66 → 67` (the new `return_scalar`); distinct annotated rules `28 → 29`. Same accept set (no grammar acceptance change — only `condition_text`'s emitted shape). Gate-locked by the `condition_text` `return_scalar` entry in `systemverilog_preprocessor_v1.json`. |
| 3 | 1.0.3 (also spans 1.0.4) | **`1.0.4` addendum — `SVPP-0002` macro-comment correctness fix (schema-neutral, non-breaking).** `macro_default_text` / `macro_body_text` were `:= inline_trivia /[^`(),?:\r\n]+/` — the content regex was **not comment-aware**, greedily eating a `/*` then halting at a backtick *inside* the `block_comment`, so valid SystemVerilog with a backtick inside a comment in a macro body / function-macro default (e.g. `` `define X a /*`*/ ``) was wrongly rejected at `1.0.3`. Fixed in `1.0.4` by making both rules comment-aware (`/(?:\/\*([^*]\|\*+[^*\/])*\*+\/\|[^`(),?:\r\n])+/` — the proven `systemverilog.ebnf` `timeunit_separator_trivia`/`block_comment` idiom): a `/* … */` comment is matched atomically (its internal backtick no longer splits the run); the unchanged `[^`(),?:\r\n]` branch still excludes a bare backtick. **No schema bump**: the rules are un-annotated, the annotation inventory is unchanged (66/28), and every input that parsed at `1.0.3` yields a byte-identical AST at `1.0.4` — only previously-*erroring* inputs now succeed with the standard `{kind:"text", body:$1}` shape (strictly more permissive). Tracked `SVPP-0002` (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`); locked by the `macro_body_comment_backtick` sample in `systemverilog_preprocessor_v1.json`. **`1.0.3` (original schema-`3` change — `macro_formals` Category-A AST-shape correction, POST-SV-AUDIT, consumer-visible):** `macro_formals` no longer exposes the raw `{first, rest}` iteration envelope — the POST-SV-AUDIT.2.1 audit (`PGEN-POST-SV-AUDIT-0002`) found `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> {first: $2, rest: $3}` was a static-conclusive Category-A raw-envelope misuse: `rest` surfaced the raw `[[comma, macro_formal], …]` separator envelope, forcing every consumer to walk past the `comma` separator. Corrected to the canonical extraction-spread `macro_formals := lparen macro_formal (comma macro_formal)* rparen -> [$2, $3::2*]` (drop the semantically-irrelevant `comma`; emit a clean flat `macro_formal` list — the `object_properties` reference idiom). For input `` `define M(a, b, c) a+b+c `` `pp_define.formals` was the raw `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}` envelope; it is now the clean list `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]` of `macro_formal` `{name, default}` objects. No `<invalid_sequence_access>` (this is a clean Category-A shape improvement, **not** the inline-alternation-`$N` corruption class of `SVPP-0001`). Surface counts **unchanged** (66 annotations / 28 distinct rules): `macro_formals` is still one rule / one annotation — only its `annotation_type` changed `return_object` → `return_array` and `normalized_text` `{first: $2, rest: $3}` → `[$2, $3::2*]`, so the surface is now **65 `return_object` + 1 `return_array`** (was all 66 `return_object`). Same accept set (no grammar acceptance change — only the annotation form). Gate-locked. |
| 2 | 1.0.2 | **SVPP-0001 correctness fix (breaking).** `pp_if_branch.keyword` no longer emits the malformed `"<invalid_sequence_access>"` object for `` `ifdef`` / `` `ifndef`` conditional input. The inline alternation `(kw_ifdef \| kw_ifndef)` that was the lead element of `pp_if_branch` (and corrupted the positional model so the bare `keyword: $1` mis-recursed) is lifted into a **named** rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} \| kw_ifndef -> {kind: "ifndef"}`, mirroring the proven `systemverilog.ebnf` op-chain / `rtl_const_expr` RTL-CE-Slice-2 idiom. `pp_if_branch`'s annotation is **unchanged** (`{keyword: $1, macro: $2, tail: $3, items: $5}`); only `$1` now binds the clean named rule, so `if_branch.keyword` is now `{kind: "ifdef"}` (or `{kind: "ifndef"}`) — a real typed polarity discriminator. Annotation count `64 → 66` (the 2 new `pp_if_keyword` `return_object` branches); distinct rules `27 → 28` (the new `pp_if_keyword`). All annotations remain `return_object`. Same accept set (no grammar acceptance change — purely the alternation lift + its 2 branch annotations). Gate-locked. |
| 1.0.0 | 1.0.1 | **SVPP-Slice-1** — initial 64-annotation baseline. pp_item dispatch (10 kinds), 7 directive shapes (define/undef/include/timescale/default_nettype/celldefine/endcelldefine), include_path/nettype_value/time_literal, conditional-compilation tree (5 nodes), condition_expr/condition_atom (12 kinds), macro_formals/formal/default_value/default_atom (8 kinds) / body/body_fragment (9 kinds), passthrough lines. **NOTE:** the `pp_if_branch.keyword` shape in this baseline was defective (`SVPP-0001`, the inline-alternation-`$N` `"<invalid_sequence_access>"` malformation) — see schema `2` for the correction. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/systemverilog_preprocessor.ebnf`) with the `systemverilog_preprocessor_file -> {type, items}` root only. AST dump is the recursive-envelope shape across all other rules. |

## Resolved Defects — `SVPP-0001` (fixed in release 1.0.2, schema 2)

- **`SVPP-0001` — `pp_if_branch.keyword` `<invalid_sequence_access>`
  (`Released`, fixed in parser release `1.0.2` / schema `2`).**
  *Historical (release `1.0.1`, schema `1`):* for `` `ifdef`` /
  `` `ifndef`` conditional input, `items[].body.if_branch.keyword`
  surfaced a malformed nested object containing three
  `"<invalid_sequence_access>"` strings instead of the keyword token.
  Root cause: `pp_if_branch := (kw_ifdef | kw_ifndef) macro_name … ->
  {keyword: $1, …}` bound `$1` to an **inline alternation group**, the
  same emit-time defect class fixed for `rtl_const_expr` in
  RTL-CE-Slice-2 (and tracked for `rtl_frontend` / `vhdl`
  `binop_chain`). The `` `define`` / non-conditional surface was
  unaffected. **Fix (proven RTL-CE-Slice-2 / `systemverilog.ebnf`
  idiom):** the inline alternation is lifted into a **named** rule
  `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef -> {kind:
  "ifndef"}`; `pp_if_branch`'s annotation is unchanged
  (`{keyword: $1, …}`) but `$1` now binds the clean named rule. The
  parser is regenerated and the corrected shape is machine-locked. For
  input `` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n`` the
  fixed `if_branch.keyword` is now `{kind: "ifdef"}` (would be
  `{kind: "ifndef"}` for an `` `ifndef`` guard) — a real typed polarity
  discriminator with **no** `<invalid_sequence_access>` anywhere.
  Consumers read the conditional polarity directly from
  `if_branch.keyword.kind`; the guard macro stays at the outer
  `if_branch.macro`. The honest pre-fix history is kept in the
  [Schema Versioning](#schema-versioning) table (schema `1.0.0` row) and
  the conditional worked example's schema-`2` transition note; tracked
  (status `Released`) in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
  (`SVPP-0001`). Documented in
  [the conditional worked example](../systemverilog_preprocessor_parser_book/src/examples-conditional.md).

## Resolved Defects — `SVPP-0002` (fixed in release 1.0.4, schema unchanged 3)

- **`SVPP-0002` — macro body / default-value content rules were not
  comment-aware; valid SystemVerilog with a backtick inside a block
  comment in a macro body or function-macro default was wrongly
  rejected (`Released`, fixed in parser release `1.0.4`, AST-dump
  schema **unchanged** `3`).**
  *Historical (releases `1.0.1`–`1.0.3`):* `macro_default_text` and
  `macro_body_text` were `:= inline_trivia /[^`(),?:\r\n]+/`. That
  content regex excludes a backtick **and is not comment-aware**, so
  it greedily consumed a comment's opening `/*` then halted at a
  backtick *inside* the `block_comment`, splitting it; nothing could
  resume at the dangling `` `*/ ``, so `macro_body+` /
  `macro_default_value+` ended short and `pp_define` could not reach
  `newline`. Repro (against `1.0.3`):
  `printf '`define X a /*`*/\n' | parseability_probe --parse
  systemverilog_preprocessor /dev/stdin` → "Parser did not consume
  full input". The byte-identical input without the backtick parsed;
  the defect did not reproduce outside the macro body/default region.
  A comment is lexically transparent, so this was **valid SV wrongly
  rejected**. Pre-existing (the rules predate `SVPP-0001` /
  POST-SV-AUDIT.2.1; not campaign-caused — proven generatively inert
  by `PGEN-SV-EXH-PROOF-0005`/`-0006`); surfaced by the SV-EXH-PROOF.2.3
  preprocessor closed-loop.
  *Fixed (`1.0.4`):* both rules made comment-aware —
  `/(?:\/\*([^*]|\*+[^*\/])*\*+\/|[^`(),?:\r\n])+/` (the proven
  `systemverilog.ebnf` `timeunit_separator_trivia` / `block_comment`
  idiom, no lookahead). A `/* … */` comment is matched **atomically**
  by the leading alternative; the unchanged `[^`(),?:\r\n]` branch
  still excludes a bare backtick (so `macro_token_paste` /
  `macro_stringize` / `macro_reference` still split real
  `` ` ``-tokens). Accepts strictly more, narrows nothing; the rules
  are un-annotated so the annotation inventory is unchanged (`66`/`28`)
  and every previously-parseable input yields a byte-identical AST —
  hence **no schema bump** (schema `3` spans `1.0.3`–`1.0.4`). Tracked
  (status `Released`) in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0002`);
  locked by the `macro_body_comment_backtick` sample in
  `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
  and `make -C rust SHELL=/opt/homebrew/bin/bash
  systemverilog_preprocessor_parser_book_gate`.

## Resolved Defects — `SVPP-0003` (fixed in release 1.0.6, schema unchanged 4)

- **`SVPP-0003` — the `` `ifdef``/`` `elsif`` condition content rule
  `condition_text` was not comment-aware; a valid SystemVerilog
  condition containing a block comment with an operator char or
  backtick inside it was wrongly rejected, and the enclosing
  `pp_conditional` then could not close (`Released`, fixed in parser
  release `1.0.6`, AST-dump schema **unchanged** `4`).** This is the
  direct sibling of `SVPP-0002`: `condition_text` was simply missed by
  the SV-EXH-PROOF.2.3.1 comment-awareness slice that fixed
  `macro_body_text` / `macro_default_text`.
  *Historical (releases `1.0.1`–`1.0.5`):* `condition_text` was
  `:= inline_trivia /[^`(),?:!|&\r\n]+/`. That content regex admits
  `/` and `*` but is **not comment-aware**, so it greedily consumed a
  comment's opening `/*` then halted at an operator char
  (`(`,`)`,`?`,`:`,`!`,`|`,`&`) or backtick *inside* the
  `block_comment`. `condition_expr := condition_atom+` could not span
  the comment (no atom matches a bare operator char mid-run), so the
  `` `elsif`` failed and the outer `pp_conditional` never reached its
  `` `endif`` closer. Repro (against `1.0.5`):
  `printf '`ifdef A\n`elsif x /*|*/\n`endif\n' | parseability_probe
  --parse systemverilog_preprocessor /dev/stdin` → "Parser did not
  consume full input". The byte-identical input with a comment that has
  **no** operator char (`/*z*/`) parsed. A comment is lexically
  transparent, so this was **valid SV wrongly rejected**. Surfaced by
  the `GRAMMAR-WELLFORMED.H.5.5` certificate-coverage multi-seed sweep
  (the stimuli generator as a bug-finding oracle): `sample_parse_failures`
  was `1` at seeds 3/6/10/12/14/15 of the 0–15 sweep, the canonical
  seed 0 being clean (so the project gates passed).
  *Fixed (`1.0.6`):* `condition_text` made comment-aware with the
  proven `SVPP-0002` idiom —
  `/(?:\/\*([^*]|\*+[^*\/])*\*+\/|[^`(),?:!|&\r\n])+/` (the `/*…*/` alt
  tried FIRST so a `/*` opener is consumed atomically; the unchanged
  `[^`(),?:!|&\r\n]` branch still excludes the operator/backtick chars
  so they still split as their own `condition_atom`s) — **and dropping
  the rule's now-redundant leading `inline_trivia`**. The drop is the
  decisive second half: with the comment-aware regex alone, the
  generator could emit a comment-only `condition_text`, but the rule's
  own leading `inline_trivia` re-consumed it as trivia on reparse,
  stranding a comment-only condition the parser could not re-derive
  (`condition_expr` needs ≥1 atom). The comment-alt + char-class
  already admit comments/whitespace, so dropping `inline_trivia` makes
  generation round-trip self-consistent without changing the `$text`
  matched-SPAN of any previously-parseable input (`` `elsif abc`` →
  `body:" abc"`, leading space preserved). Accepts strictly more,
  narrows nothing; `condition_text` stays `-> $text`, the annotation
  inventory is unchanged (`67`/`29`), and every previously-parseable
  input yields a byte-identical AST — hence **no schema bump** (schema
  `4` spans `1.0.5`–`1.0.6`). Tracked (status `Released`) in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0003`);
  locked by the `condition_comment_special_char` sample in
  `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`,
  the svpp certificate-coverage `sample_parse_failures=0` across seeds
  0–31, and `make -C rust SHELL=/opt/homebrew/bin/bash
  systemverilog_preprocessor_parser_book_gate`.

## Resolved Defects — `SVPP-0004` (fixed in release 1.0.7, schema 4 → 5)

- **Defect:** a macro default argument on the **last** formal mis-parsed. `` `define M(a=x) y`` — legal
  IEEE 1800 §22.5.1 (the LRM's own examples place defaults on the last formal) — produced
  `formals: []` with the entire `(a=x) y` routed into the macro *body* as fragments
  (`lparen` / text `"a=x"` / `rparen` / text `"y"`), i.e. a wrong-but-accepted AST. A default on a
  NON-last formal (`` `define M(a=x,b) y``) parsed correctly, which is why the defect hid.
- **Root cause:** `macro_default_value := macro_default_atom+` is possessive (PEG), and
  `macro_default_atom` carried bare `lparen`/`rparen` alternatives (meant for nested parens inside
  default text). The atom run therefore swallowed the FORMALS' closing `)`;
  `macro_formals := lparen … rparen` could not close; the `macro_formals?` optional backtracked to
  empty; the parenthesized text re-parsed as macro body. Per the LRM, default text may contain a `,`
  or `)` only "inside a balanced pair of parentheses" — the grammar lacked the balanced-pair model.
- **Fix (grammar-only):** `macro_default_paren_group := lparen macro_default_group_atom* rparen ->
  {kind: "paren_group", atoms: $2}` with `macro_default_group_atom := macro_default_atom -> $1 |
  comma -> {kind: "comma"}` replaces the bare paren alternatives. An unbalanced `)` now correctly
  terminates the default; commas are legal inside the group (and only there); nesting balances
  recursively.
- **How it was found:** the `GRAMMAR-WELLFORMED.H.7.2` plannable-rule reach pass (the stimuli
  generator as a bug-finding oracle) — its probes for the never-witnessed `macro_default_text` rule
  kept re-parsing with the bytes routed elsewhere (`parsed-but-routed-elsewhere`), and chasing that
  verdict produced the decisive AST repro pair.
- **Verification:** repro pair correct (`a=x` last-formal now structured formals; `a=x,b` byte-stable);
  `` `define M(a=(x,y)) z`` → `paren_group{text "x", comma, text "y"}` (the LRM balanced-pair model,
  exactly); svpp certificate-coverage sweep `fully_certified=true` + `sample_parse_failures=0` at ALL
  32 seeds 0–31; `sv_preprocessor_zero_plausible_gap_proof_gate` GREEN; svpp AST shape-contract GREEN
  with the new `macro_default_on_last_formal` sample; full dual-feature lib green; cross-grammar
  cert-coverage untouched (svpp-only edit).
- **Consumer impact:** breaking ONLY for consumers that relied on the two buggy shapes (empty-formals
  body-text for default-on-last-formal; flat `lparen`/`rparen` atoms in paren-bearing defaults) —
  hence schema `4 → 5`. All other inputs are byte-identical.

## Editions and Profiles (release 1.0.9)

From release `1.0.9` the parser reads compiler directives the way a named IEEE edition reads them
(`SV-CORPUS-GRAD.13c.2v.9`, by director ruling 2026-10-18: *"give the grammar per-edition profils"*). The profiles are
the SystemVerilog family's own names, so a consumer hands this parser the profile it hands the SystemVerilog parser:

| profile | edition | request spellings (`@profile_alias`) |
|---|---|---|
| `sv_2017` — **the default** (`@default_profile`) | IEEE 1800-2017 clause 22 | `2017`, `ieee1800-2017`, `ieee_1800_2017` |
| `sv_2023` | IEEE 1800-2023 clause 22 | `2023`, `ieee1800-2023`, `ieee_1800_2023` |
| `sv_2023_tolerant` | as `sv_2023` (no directive is tolerated beyond 1800-2023) | — |
| `verilog_2005` | IEEE 1364-2005 clause 19 | `1364-2005`, `ieee1364-2005`, `ieee_1364_2005` |

An unspecified profile reads IEEE 1800-2017, as the expansion stage's unprofiled reading does; the generated parser
carries `DEFAULT_GRAMMAR_PROFILE = "sv_2017"`, and `set_grammar_profile(None)` restores it.

**What differs** — every row a reproducer in the census bank (`docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/bank/`,
read under each profile it splits), every rule cited to the in-repo clause in `docs/tasks/SV-CORPUS-GRAD.md` leaf
`.13c.2v.13`:

| the form | `verilog_2005` | `sv_2017` | `sv_2023` |
|---|---|---|---|
| `` `ifdef ( expr ) `` / `` `elsif ( expr ) `` (`! && \|\| -> <->`) | refused | refused | derived |
| a triple-quoted string `"""…"""` | refused | refused | derived |
| `version_specifier` | the four `1364-*` | adds `1800-2005` … `1800-2017` | adds `1800-2023` |
| a directive's tokens over several lines (a formal list included; a block comment crossing a line inside the directive counts) | derived | derived | refused (§22.2 *"all on one line"*) |
| a block comment in macro text crossing a line without a backslash | refused | refused | derived |
| an escaped macro name's formals after its terminating white space | macro text | macro text | formals |
| `` `include <file> ``, formal defaults, `` `" `` / `` `\`" `` / ``` `` ``` | refused | derived | derived |
| `` `undefineall `` | a macro usage | the directive | the directive |
| a directive's name as a macro name | refused (its 19) | refused (its 22) | refused (its 22) |

Identical in every profile: `` `pragma ``, `` `default_nettype `` (Syntax 22-7's eleven values), `` `line ``, `` `undef ``,
`` `include ``'s line rule, and a macro usage in any directive parameter (a defined macro *"can be used anywhere"*).

## Resolved Defects — `SVPP-0010` … `SVPP-0015`, and the ordered gaps (fixed in release 1.0.9, schema 6 → 7)

| ledger | the `1.0.8` behaviour | `1.0.9` |
|---|---|---|
| `SVPP-0010` | a block comment opened after code on its line was not opaque — a directive word on a following line inside it was read as a directive (Surelog `tests/PPComment/dut.sv`) | passthrough text is lexed; the comment hides it |
| `SVPP-0011` | a passthrough line was not lexed — a string left open at its line end passed (seven suite error tests) | an unclosed string or comment is refused in every group |
| `SVPP-0012` | text after `` `ifdef X `` / `` `else `` / `` `endif `` on its line was the directive's `tail`, after `` `elsif X `` part of its condition | the group's first item; `tail` holds a trailing comment only |
| `SVPP-0013` | a formal list that did not parse fell back to an OBJECT-LIKE define (`(typ,` as text) — multi-line lists, an empty default (UVM's `uvm_declare_type_alias(TYPE,NAME,SFX=)`), a comment between formals | a `(` right after the name always opens the list; newlines and empty defaults per edition; a comment there is refused |
| `SVPP-0014` | the punctuation terminals were string terminals whose default skip crossed newlines and comments — a define body ran on into a next line opening with `(` | every punctuation terminal is a regex; a define ends at its line end |
| `SVPP-0015` | a macro usage's actual arguments inside a directive parameter were plain runs between balanced parentheses: `` `include `M(")") `` was refused, a `)` inside a comment closed the list, and `` `M("unclosed) `` passed | the actuals are lexed per §22.5.1: a string, a block comment and an escaped identifier are atoms, a parenthesis inside `[ ]` / `{ }` is a character; an open string is refused |

And the gaps release `1.0.8` listed as ordered fixed: a conditional written on one line is ACCEPTED (`.13c.2v.10`); a
line inside a conditional group that does not parse is RECORDED as a `malformed_directive` item, not refused — liveness
is the expansion stage's to decide (`.13c.2v.10`); the per-edition profiles above (`.13c.2v.9`); a macro name may be an
escaped identifier and may not be a directive's name (`.13c.2v.8`). A directive may begin anywhere on a line (1800-2023
§22.2), except `` `include `` and `` `line ``.

**Found before the release, fixed in it** (`SV-CORPUS-GRAD.13c.2v.16`: the certificate's generated samples, which reach
forms no corpus file holds, refused where the first `1.0.9` build did): a define with no macro text that ENDS THE FILE
after white space or a comment (`` `define A `` with a trailing space and no newline) is accepted, as `1.0.8` did; a
formal default that is ONLY a comment (`` `define M(a=/*c*/) x ``) is a function-like define's default — `1.0.8`
accepted it only through `SVPP-0013`'s object-like fallback, with `formals: []`; and under `sv_2023` a block comment
that crosses a line inside a directive — between its tokens, in a formal default, in a `` `pragma `` — is refused, as
the edition table above says (the first build let a comment carry the directive onto a second line).

**Measured on the generated parser** (`census.py`, the release probe, against the `1.0.8` probe): the default reading (IEEE 1800-2017) over the 16,336-file SystemVerilog corpus — `fail → pass 14`, `pass → fail 38` (the suites' own error tests, strings split between a macro and its usage, `` `" `` / ``` `` ``` outside macro text, comments between formals, an empty formal list, two 1800-2023 forms — none valid under its reading), 88 refusals each adjudicated in `residual_dispositions.tsv`, 82 previously accepted trees moved, each attributed; the `sv_2023` reading 95 refusals, the `verilog_2005` reading over its own 2,943-file lane 56, each adjudicated in its own table; the bank 119 / 119 (19 cases under a named profile); the error-test half: of the 82 preprocessor-themed suite error tests the default reading refuses 47, and each of the 35 it accepts carries a written reason (`error_test_dispositions.tsv`).

## Resolved Defects — `SVPP-0005` … `SVPP-0009` (fixed in release 1.0.8, schema 5 → 6)

- **How they were found:** the first corpus reading this parser has had (`SV-CORPUS-GRAD.13c.2v.2`,
  `docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py`): the GENERATED parser inside the release
  `parseability_probe`, run over the 16,336 files the SystemVerilog lanes read — every one a preprocessor input
  whatever its SystemVerilog verdict. Release `1.0.7` refused **4,477** of them.
- **`SVPP-0005` — valid clause-22 text refused.** A passthrough line could not BEGIN with a backtick
  (`` non_directive_text := inline_trivia /[^`\r\n][^\r\n]*/ ``), so a line opening with a text macro usage
  (§22.5.1 Syntax 22-3; `` `__FILE__ `` / `` `__LINE__ `` are macros, §22.13) was refused — 4,182 files; seven
  directives had no production (§22.3 `` `resetall ``, §22.5.3 `` `undefineall ``, §22.9 `` `unconnected_drive `` /
  `` `nounconnected_drive ``, §22.11 `` `pragma ``, §22.12 `` `line ``, §22.14 `` `begin_keywords `` /
  `` `end_keywords ``) — 243 files; and `` `include `MACRO `` (§22.5.1, both editions: *"The `` `include ``
  directive can be followed by a macro, instead of a string literal"*) was refused. **Fix:** eight `pp_item` kinds,
  an `include_path` `macro` kind, and `pp_non_directive_line := !directive_keyword non_directive_text newline?` with
  a backtick-identifier alternative — the lookahead over the twenty §22.1 keywords is what keeps a MALFORMED
  directive (`` `timescale 1ns ``, a stray `` `else ``) refused.
- **`SVPP-0006` — a continued define body cut at its first line.** `` `define M(a) a + \ `` + `  b` gave the body
  `text "a + \\"` and a separate `non_directive_line` for `  b` (§22.5.1: the newline *"shall be preceded by a
  backslash"*). **Fix:** `{kind: "line_continuation"}` in `macro_body_fragment`; the text run consumes a backslash
  only with the character after it. 38 corpus trees.
- **`SVPP-0007` — `` `include "a.sv" x `` accepted.** §22.4 (both editions): *"Only white space or a comment may
  appear on the same line as the `` `include `` compiler directive"*. **Fix:** `pp_include` requires the end of the
  line (`line_end := /\r?\n|\z/`). No corpus file moved.
- **`SVPP-0008` — `` `define M (x) y `` read as function-like.** §22.5.1 (2017): *"The left parenthesis shall follow
  the text macro name immediately, with no space in between"* — the macro is object-like, with the text `(x) y`; the
  expansion stage always read it so. **Fix:** `formals_open := /\(/`, a regex token that skips nothing.
- **`SVPP-0009` — macro text refused or split wrongly.** No `` `\`" `` arm; a plain string literal was not an atom,
  so a backtick inside it ended the text run and `(` `)` `,` `?` `:` split it, though *"Macro substitution and
  argument substitution shall not occur within string literals"*; a backtick in a `//` comment ended the run; a
  quoted comma in a default broke the formals. **Fix:** `{kind: "escaped_quote"}`; block comments, `//` comments
  (up to a continuation backslash) and plain strings are atoms of `macro_body_text`; strings of `macro_default_text`.
  31 corpus trees moved, each at a string literal.
- **Verification:** `census.py --probe` (release `1.0.8`) `--before` (the `1.0.7` probe): rejections
  **4,477 → 64**, **zero `pass → fail`**, every residual adjudicated in `residual_dispositions.tsv` (34 suite error
  tests, 16 refused by a clause, 12 valid text routed to `SV-CORPUS-GRAD.13c.2v.8` / `.9` / `.10`, 2 malformed directives inside a skipped group —
  every one of the 14 a known gap ordered fixed, see [Known Gaps](#known-gaps--release-108-each-ordered-fixed-none-is-a-limit-of-this-contract)); 11,790 of the 11,859 previously accepted trees byte-identical and the 69 that moved
  each attributed (38 continuation, 31 string atom), the same sets the interpreter measured on the prototype before
  the build; the clause-22 bank `census.py --bank` 66/66 (42 ACCEPT, 24 REJECT); the certificate sweep
  `fully_certified=true UNKNOWN=0 sample_parse_failures=0` at all 32 seeds 0–31 (117 rules: 115 witnessed, 2 proven);
  the shape manifest's thirteen samples, seven of them JSON-pointer pins of the trees above.
- **Consumer impact:** additive for the new kinds; breaking for the four corrected classes and for any consumer that
  dispatched `pp_item` / `macro_body_fragment` by branch INDEX rather than `kind` — hence schema `5 → 6`.

## AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT) — `macro_formals` Category-A raw-envelope → clean list; schema 2 → 3

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.1, tracked
`PGEN-POST-SV-AUDIT-0002`) found one static-conclusive Category-A
raw-envelope misuse in `grammars/systemverilog_preprocessor.ebnf`. It
is corrected, the parser is regenerated, and the manifest inventory is
re-locked. This is **not** a released-parser bug (no
`<invalid_sequence_access>`, no crash) — it is a deliberate
audit-driven AST-shape correction, so it carries a schema bump but is
**not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
(that ledger is reserved for the `<invalid_sequence_access>`
corruption/crash class — `SVPP-0001` et al.).

- **`macro_formals` raw `{first, rest}` envelope (Category-A,
  `PGEN-POST-SV-AUDIT-0002`).** For a parameterised `` `define``,
  `pp_define.formals` is the typed `macro_formals` object. The
  `1.0.1`/`1.0.2` grammar was

  ```ebnf
  macro_formals := lparen macro_formal (comma macro_formal)* rparen
                -> {first: $2, rest: $3}
  ```

  which bound `rest` to the **raw iteration envelope**
  `[[comma, macro_formal], …]` — every consumer had to index past the
  `comma` separator on each iteration to reach the next `macro_formal`.
  This is a static-conclusive Category-A pure-single-token-separator
  list misuse (the `comma` carries no payload a consumer needs). **Fix
  (the canonical `object_properties` extraction-spread idiom):** drop
  the separator and emit a clean flat list:

  ```ebnf
  macro_formals := lparen macro_formal (comma macro_formal)* rparen
                -> [$2, $3::2*]
  ```

  `macro_formals` is still one rule with one annotation — only its
  annotation form changed (`annotation_type` `return_object` →
  `return_array`; `normalized_text` `{first: $2, rest: $3}` →
  `[$2, $3::2*]`). Real captured before→after for input
  `` `define M(a, b, c) a+b+c `` (verified via `parseability_probe`), at
  `pp_define.formals`:

  - **Before (≤ schema 2 / release 1.0.2):**
    `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}`
  - **After (schema 3 / release 1.0.3):**
    `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]`
    — a clean flat list of `macro_formal` `{name, default}` objects, in
    source order. No `<invalid_sequence_access>` anywhere (this is a
    clean Category-A shape improvement, **not** the inline-alternation
    corruption class of `SVPP-0001`).

  Consumers now iterate `pp_define.formals` directly as the
  `macro_formal` array — no `.first` / `.rest` split, no separator to
  skip. A consumer written against `1.0.2` that walked
  `formals.first` + `formals.rest[][1]` must repin to schema `3` and
  treat `formals` as a flat `macro_formal[]`.

Annotation count: **66** (unchanged — `macro_formals` was, and remains,
one rule / one annotation; no count delta). **28** distinct rules
(unchanged). The annotation-type mix is now **65 `return_object` + 1
`return_array`** (the single `return_array` is `macro_formals`; it was
the only `return_object` → `return_array` change). Same accept set (no
grammar acceptance change — purely the annotation-form change). Schema
bumped `2 → 3` because `pp_define.formals` (`macro_formals`) changed
shape in a consumer-visible way. Gate-locked:
`cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract`
and
`make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`.

> **Audit-campaign note:** POST-SV-AUDIT.2.1 is the first Category-A
> fix of the POST-SV-AUDIT.2 worklist
> (`docs/POST_SV_AUDIT_LEDGER.md`). The remaining Category-A
> raw-envelope list rules in `rtl_frontend.ebnf`, `vhdl.ebnf`, and
> `systemverilog.ebnf` are tracked separately as their own
> POST-SV-AUDIT.2.x slices; this release corrects
> systemverilog_preprocessor only. Distinct from the `SVPP-0001`
> inline-alternation-`$N` corruption class — that was a released-parser
> bug (`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`); this
> Category-A correction is not.

## Release 1.0.2 / Contract 1.0.2 Highlights — SVPP-0001 correctness fix (pp_if_branch.keyword); schema 1 → 2

Landed 2026-05-16. A worked-example pass surfaced that the `1.0.1`
baseline shipped one return-annotation defect (`SVPP-0001`) that the
(root-keys-only) shape-contract regression lock did not catch. It is
fixed, the parser is regenerated, and the manifest inventory is
tightened to the full 66-entry surface so the corrected shape is now
machine-locked.

- **`pp_if_branch.keyword` `<invalid_sequence_access>` (SVPP-0001).**
  For any `` `ifdef`` / `` `ifndef`` conditional input,
  `items[].body.if_branch.keyword` emitted a malformed nested object
  containing three `"<invalid_sequence_access>"` strings instead of the
  keyword token. Root cause:
  `pp_if_branch := (kw_ifdef | kw_ifndef) macro_name directive_tail?
  newline pp_item* -> {keyword: $1, macro: $2, tail: $3, items: $5}`
  bound `$1` to an **inline alternation group**
  `(kw_ifdef | kw_ifndef)`, which corrupts the positional model so the
  rule's own annotation is mis-recursed. **Fix (proven RTL-CE-Slice-2 /
  `systemverilog.ebnf` op-chain idiom):** the inline alternation is
  lifted into a **named** rule:

  ```ebnf
  pp_if_keyword := kw_ifdef  -> {kind: "ifdef"}
                 | kw_ifndef -> {kind: "ifndef"}
  pp_if_branch  := pp_if_keyword macro_name directive_tail? newline pp_item*
                -> {keyword: $1, macro: $2, tail: $3, items: $5}
  ```

  `pp_if_branch`'s annotation is **unchanged** — only `$1` now binds
  the clean named `pp_if_keyword` rule instead of the inline group. The
  fixed shape for input
  `` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n`` is
  `pp_conditional` = `{if_branch, elsif_branches, else_branch}`;
  `if_branch` = `{keyword: {kind: "ifdef"}, macro: [[" "], "X"],
  tail: [], items: [{kind: "define", …}]}`. `if_branch.keyword` is now
  the typed polarity object `{kind: "ifdef"}` (or `{kind: "ifndef"}`) —
  **no** `<invalid_sequence_access>` anywhere. Consumers read the
  `` `ifdef`` vs `` `ifndef`` polarity directly from
  `if_branch.keyword.kind`; the guard macro stays at the outer
  `if_branch.macro`.

Annotation count: **66** (was 64; +2 = the new `pp_if_keyword`
`return_object` branches `{kind: "ifdef"}` / `{kind: "ifndef"}`). **28**
distinct rules (was 27; +1 = `pp_if_keyword`). All 66 remain
`annotation_type: "return_object"`. Same accept set (no grammar
acceptance change — purely the alternation lift + its 2 branch
annotations). Schema bumped `1 → 2` because `pp_if_branch.keyword`
changed shape in a consumer-visible way (was the malformed
`<invalid_sequence_access>` object, now the typed `{kind}` polarity
discriminator). Gate-locked:
`cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract`
and
`make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate`.

> **Systemic note:** the inline-operator-alternation antipattern that
> caused `SVPP-0001` also exists in `grammars/rtl_frontend.ebnf` and
> `grammars/vhdl.ebnf` `binop_chain` levels (same
> `<invalid_sequence_access>` empirically confirmed for rtl_frontend
> `a + b`), and was the same root cause fixed for `rtl_const_expr` in
> RTL-CE-Slice-2 (`docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`
> Release 1.0.2 systemic note). Those families' corrections are tracked
> separately as their own slices + bug-ledger entries; this release
> fixes systemverilog_preprocessor only.

## Release 1.0.1 / Contract 1.0.1 Highlights — SVPP-Slice-1: full grammar typed (40+ rules / 63 annotations)

Single comprehensive slice landed on 2026-05-14 covering the entire grammar surface:

```ebnf
# File root (pre-existing)
systemverilog_preprocessor_file  -> {type: "systemverilog_preprocessor_file", items}

# Dispatch wrapper (10 kinds)
pp_item                          -> {kind: "define" | "undef" | "include" | "timescale"
                                          | "default_nettype" | "celldefine" | "endcelldefine"
                                          | "conditional" | "non_directive_line" | "blank_line",
                                     body?}

# Per-directive shapes (7)
pp_define                        -> {name, formals, body}
pp_undef                         -> {name, comment}
pp_include                       -> {path, comment}
pp_timescale                     -> {unit, precision, comment}
pp_default_nettype               -> {nettype, comment}
pp_celldefine                    -> {comment}
pp_endcelldefine                 -> {comment}

# Include path + nettype (2 kinds each)
include_path                     -> {kind: "quoted"|"angle", text}
nettype_value                    -> {kind: "identifier"|"none", body?}
time_literal                     -> {value, unit}

# Conditional compilation (5 nodes)
pp_conditional                   -> {if_branch, elsif_branches, else_branch}
pp_if_branch                     -> {keyword, macro, tail, items}
pp_elsif_branch                  -> {condition, items}
pp_else_branch                   -> {tail, items}
pp_endif                         -> {tail}

# Condition expression (12-kind atom)
condition_expr                   -> {atoms}
condition_atom                   -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"comma"|"question"|"colon"
                                          |"logical_or"|"logical_and"|"bang", body?}

# Macro formals + default values (8-kind atom)
macro_formals                    -> {first, rest}   # 1.0.1/1.0.2 shape; corrected to [$2, $3::2*] (clean macro_formal list) in 1.0.3 / schema 3 — see "AST-Shape Corrections — 1.0.3"
macro_formal                     -> {name, default}
macro_default_value              -> {atoms}
macro_default_atom               -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"question"|"colon", body?}

# Macro body fragment (9 kinds)
macro_body                       -> {fragments}
macro_body_fragment              -> {kind: "token_paste"|"stringize"|"macro_reference"|"text"
                                          |"lparen"|"rparen"|"comma"|"question"|"colon", body?}

# Passthrough lines
pp_non_directive_line            -> {text}
pp_blank_line                    -> {kind: "blank"}
```

Annotation count: **64** (was 1 / foundation baseline). Same accept set.

## AST Envelope and pp_item Dispatch

This section is the consumer-facing dispatch contract: how a downstream
integrator goes from the host AST-dump call to a typed
systemverilog_preprocessor tree, and how to branch on the top-level
discriminators. Every shape below is transcribed from the live inventory
`generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 66`, **28 distinct rules**), cross-checked against the
embedded copy in
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
(content-identical on the `(rule, branch_index, annotation_type,
normalized_text)` tuples; the embedded copy omits only the diagnostic
`raw_text` field), and is consistent with the curated per-rule reference
at `docs/systemverilog_preprocessor_parser_book/src/rules-top-level.md`.

### The `AstDumpPayload` envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`, used with grammar family
`systemverilog_preprocessor` / profile `default`) return — on success —
an `AstDumpPayload` (defined in `rust/src/embedding_api.rs`, contract in
`rust/docs/EMBEDDING_API_CONTRACT.md`). It is a canonical-JSON payload
string plus truncation metadata, with exactly four fields:

| Field | Type | Meaning |
|---|---|---|
| `dump_json` | string | The canonical (key-sorted) JSON encoding of the typed systemverilog_preprocessor AST. Parse this string to obtain the `systemverilog_preprocessor_file` root object described below. |
| `truncated` | bool | `false` for a complete dump; `true` when `max_ast_bytes` was exceeded and `dump_json` instead carries the truncation diagnostic envelope. |
| `full_bytes` | int | Byte length of the full encoded AST payload (before any truncation). |
| `emitted_bytes` | int | Byte length actually placed in `dump_json`. Equals `full_bytes` when not truncated. |

When `truncated` is `true`, `dump_json` is replaced by a deterministic
truncation diagnostic envelope (not the AST). That envelope carries
`pgen_dump_contract_version` (currently `1`), `kind:
"pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
"parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`. Consumers
must check `truncated` (or, equivalently, the presence of
`pgen_dump_contract_version` / `kind == "pgen_ast_dump_truncation"` in
the parsed `dump_json`) before treating `dump_json` as a
systemverilog_preprocessor AST. If `max_ast_bytes` is too small to fit
even the diagnostic envelope, the API returns `E_INVALID_LIMITS`
instead.

> Accuracy note: the live `AstDumpPayload` struct exposes precisely
> `dump_json` / `truncated` / `full_bytes` / `emitted_bytes`. The
> `pgen_dump_contract_version` / `schema_version` / `grammar` / `profile` /
> `root` keys are **not** members of `AstDumpPayload` itself —
> `pgen_dump_contract_version` appears only inside the truncation
> diagnostic envelope, the schema axis is the **AST-dump schema version
> `3`** tracked in [Schema Versioning](#schema-versioning), the grammar
> family is the fixed `systemverilog_preprocessor` label, and the profile
> is the fixed `default` profile (see [Stable Integration
> Surface](#stable-integration-surface)). The "root" is the parsed
> `systemverilog_preprocessor_file` object documented next. This contract
> documents the surface as it exists in `rust/src/embedding_api.rs`, not
> an idealized envelope. Where this prose and the inventory disagree, the
> inventory wins.

### The `systemverilog_preprocessor_file` root

The parsed `dump_json` is, for a successful systemverilog_preprocessor
parse, a single typed root object. Per
`grammars/systemverilog_preprocessor.ebnf` (lines 15–16):

```ebnf
systemverilog_preprocessor_file := pp_item*
-> {type: "systemverilog_preprocessor_file", items: $1}
```

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [ /* array of pp_item shapes, source order */ ]
}
```

Consumers dispatch on `obj["type"] == "systemverilog_preprocessor_file"`
at the root, then iterate `obj["items"]` — each element is one typed
`pp_item` object in source order. This is the only rule that carries a
`type` discriminator at the dispatch level; every other dispatcher uses
`kind`.

### The 18-branch `pp_item` dispatch

`pp_item` is the primary top-level dispatcher. It is an 18-branch
`kind`-tagged shape (10 through `1.0.7`; `1.0.8` added the eight kinds of the seven
clause-22 directives the grammar had not derived). Consumers dispatch on `obj["kind"]`; every branch except the
three bodyless ones (`"celldefine"`, `"endcelldefine"`, `"blank_line"`)
carries a `body` holding the underlying typed shape:

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

| Branch | `kind` | `body` shape (fields) | Underlying rule (`grammars/systemverilog_preprocessor.ebnf`) |
|---|---|---|---|
| 0 | `"define"` | `{name, formals, body}` — `formals` is `[]` when there is no `(...)` formal list (and, since `1.0.8`, when white space separates the name from a parenthesis — `SVPP-0008`); `body` is `[]` for a bodyless `` `define`` | `pp_define` |
| 1 | `"undef"` | `{name, comment}` | `pp_undef` |
| 2 | `"include"` | `{path, comment}` — `path` is the typed `include_path`; the line must end after it (`SVPP-0007`) | `pp_include` |
| 3 | `"timescale"` | `{unit, precision, comment}` — `unit` / `precision` are typed `time_literal` | `pp_timescale` |
| 4 | `"default_nettype"` | `{nettype, comment}` — `nettype` is the typed `nettype_value` | `pp_default_nettype` |
| 5 | `"celldefine"` | _(no `body` — bare `{kind: "celldefine"}`)_ | `pp_celldefine` |
| 6 | `"endcelldefine"` | _(no `body` — bare `{kind: "endcelldefine"}`)_ | `pp_endcelldefine` |
| 7 | `"resetall"` | `{comment}` (§22.3; `1.0.8`) | `pp_resetall` |
| 8 | `"undefineall"` | `{comment}` (§22.5.3; `1.0.8`) | `pp_undefineall` |
| 9 | `"unconnected_drive"` | `{drive, comment}` — `drive` is `{kind: "pull0"}` or `{kind: "pull1"}` (§22.9; `1.0.8`) | `pp_unconnected_drive` |
| 10 | `"nounconnected_drive"` | `{comment}` (§22.9; `1.0.8`) | `pp_nounconnected_drive` |
| 11 | `"pragma"` | `{name, expressions, comment}` — Syntax 22-8; nothing may follow on the line (§22.11; `1.0.8`) | `pp_pragma` |
| 12 | `"line"` | `{number, filename, level}` — Syntax 22-9; white space only on the line, no comment (§22.12; `1.0.8`) | `pp_line` |
| 13 | `"begin_keywords"` | `{version, comment}` — one of the nine Syntax 22-10 specifiers (§22.14; `1.0.8`) | `pp_begin_keywords` |
| 14 | `"end_keywords"` | `{comment}` (§22.14; `1.0.8`) | `pp_end_keywords` |
| 15 | `"conditional"` | `{if_branch, elsif_branches, else_branch}` — the conditional-compilation tree | `pp_conditional` |
| 16 | `"non_directive_line"` | `{text}` — a passthrough source line, including one that OPENS with a text macro usage since `1.0.8` (`SVPP-0005`) | `pp_non_directive_line` |
| 17 | `"blank_line"` | _(no `body` — bare `{kind: "blank_line"}`)_ | `pp_blank_line` |

The inventory confirms exactly these **18** `pp_item` branches (one
`return_object` annotation per branch, branch indices 0–17; dispatch on
`kind`, never on the index — the indices shifted in `1.0.8`); there is no
other `pp_item` `kind`. The three bodyless kinds are `"celldefine"`,
`"endcelldefine"`, and `"blank_line"` (their `pp_celldefine` /
`pp_endcelldefine` rules are still typed — they emit `{comment}` — but
`pp_item` discards the `body` for those two and for the
zero-information `pp_blank_line`).

### Per-directive shapes

Below the `pp_item` dispatch the fifteen non-conditional directive rules
emit named-field objects. The inventory confirms exactly **15** directive
shapes — the seven of `1.0.1` (`pp_define`, `pp_undef`, `pp_include`,
`pp_timescale`, `pp_default_nettype`, `pp_celldefine`, `pp_endcelldefine`)
and the eight of `1.0.8` (`pp_resetall`, `pp_undefineall`,
`pp_unconnected_drive`, `pp_nounconnected_drive`, `pp_pragma`, `pp_line`,
`pp_begin_keywords`, `pp_end_keywords`) — each a single-branch
`return_object`:

```ebnf
pp_define          := kw_define macro_name macro_formals? macro_body? newline?
                   -> {name: $2, formals: $3, body: $4}
pp_undef           := kw_undef macro_name directive_comment_tail newline?
                   -> {name: $2, comment: $3}
pp_include         := kw_include include_path directive_comment_tail line_end
                   -> {path: $2, comment: $3}
pp_timescale       := kw_timescale time_literal slash time_literal directive_comment_tail newline?
                   -> {unit: $2, precision: $4, comment: $5}
pp_default_nettype := kw_default_nettype nettype_value directive_comment_tail newline?
                   -> {nettype: $2, comment: $3}
pp_celldefine      := kw_celldefine directive_comment_tail newline?
                   -> {comment: $2}
pp_endcelldefine   := kw_endcelldefine directive_comment_tail newline?
                   -> {comment: $2}
pp_resetall            := kw_resetall directive_comment_tail newline?            -> {comment: $2}
pp_undefineall         := kw_undefineall directive_comment_tail newline?         -> {comment: $2}
pp_unconnected_drive   := kw_unconnected_drive pull_drive directive_comment_tail newline?
                       -> {drive: $2, comment: $3}
pp_nounconnected_drive := kw_nounconnected_drive directive_comment_tail newline? -> {comment: $2}
pp_pragma              := kw_pragma pragma_name pragma_expression_list? directive_comment_tail line_end
                       -> {name: $2, expressions: $3, comment: $4}
pp_line                := kw_line line_number line_filename line_level line_terminator
                       -> {number: $2, filename: $3, level: $4}
pp_begin_keywords      := kw_begin_keywords version_specifier directive_comment_tail newline?
                       -> {version: $2, comment: $3}
pp_end_keywords        := kw_end_keywords directive_comment_tail newline?        -> {comment: $2}
```

| Rule (`grammars/systemverilog_preprocessor.ebnf`) | Shape | Notes |
|---|---|---|
| `pp_define` (line 33) | `{name, formals, body}` | `name` is the un-annotated `macro_name`/`identifier` envelope; `formals` is `[]` when no `(...)` formal list, else the clean `macro_formal[]` list (the `1.0.3` / schema `3` POST-SV-AUDIT Category-A `[$2, $3::2*]` extraction-spread — was the raw `{first, rest}` envelope at ≤ `1.0.2` / schema `2`; see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3)); `body` is `[]` for a bodyless macro, else the typed `macro_body` `{fragments}` object. |
| `pp_undef` (line 35) | `{name, comment}` | `comment` is the `directive_comment_tail` envelope (`[]` when no trailing comment). |
| `pp_include` | `{path, comment}` | `path` is the typed `include_path` (`{kind: "quoted"\|"angle", text}` or, since `1.0.8`, `{kind: "macro", body: {name, arguments}}`). The line must end after it (§22.4; `SVPP-0007`). |
| `pp_timescale` (line 39) | `{unit, precision, comment}` | `unit` / `precision` are typed `time_literal` (`{value, unit}`). |
| `pp_default_nettype` (line 41) | `{nettype, comment}` | `nettype` is the typed `nettype_value` (`{kind: "identifier", body}` / `{kind: "none"}`). |
| `pp_celldefine` (line 43) | `{comment}` | The `pp_item` `"celldefine"` branch discards this `body`. |
| `pp_endcelldefine` (line 45) | `{comment}` | The `pp_item` `"endcelldefine"` branch discards this `body`. |
| `pp_resetall` / `pp_undefineall` / `pp_nounconnected_drive` / `pp_end_keywords` | `{comment}` | `1.0.8`; the body is kept at the dispatch. |
| `pp_unconnected_drive` | `{drive, comment}` | `drive` is `pull_drive`: `{kind: "pull0"}` / `{kind: "pull1"}`. |
| `pp_pragma` | `{name, expressions, comment}` | `name` the `pragma_name` envelope; `expressions` the `pragma_expression` list (`[]` when absent) — each `{kind: "keyword_value", keyword, value}`, `{kind: "keyword", keyword}` or `{kind: "value", value}`, a value `{kind: "list", expressions}` / `{kind: "number"\|"string", text}` / `{kind: "identifier", name}` (Syntax 22-8). The line must end after the expressions and an optional comment. |
| `pp_line` | `{number, filename, level}` | Each the `[[space], "text"]` envelope; `number` a positive integer, `level` `0`/`1`/`2` (Syntax 22-9). |
| `pp_begin_keywords` | `{version, comment}` | `version` the quoted specifier's envelope — one of 1800-2023's nine. |

The supporting leaf shapes are `include_path` (3 kinds:
`{kind: "quoted", text}` / `{kind: "angle", text}` / `{kind: "macro", body}` since `1.0.8`),
`nettype_value` (2 kinds: `{kind: "identifier", body}` /
`{kind: "none"}`, lines 51–52), and `time_literal`
(`{value, unit}`, lines 54–55).

The conditional-compilation tree (`pp_conditional` →
`{if_branch, elsif_branches, else_branch}`, line 61) descends into
`pp_if_branch` (`{keyword, macro, tail, items}`, line 64),
`pp_elsif_branch` (`{condition, items}`, line 66), `pp_else_branch`
(`{tail, items}`, line 68), and `pp_endif` (`{tail}`, line 70); each
branch's `items` is a nested `pp_item*` array. The condition / macro
formal / macro body atom dispatchers (`condition_expr` /
`condition_atom`, `macro_formals` (the clean `macro_formal[]` list as of
`1.0.3` / schema `3`) / `macro_formal` / `macro_default_value` /
`macro_default_atom`, `macro_body` / `macro_body_fragment`) and the
passthrough lines (`pp_non_directive_line` → `{text}`, `pp_blank_line`
→ `{kind: "blank"}`) round out the 28-distinct-rule typed surface; their
full per-branch field lists are in
`docs/systemverilog_preprocessor_parser_book/src/rules-top-level.md`.


**Release `1.0.9` — parameters.** Any fixed-syntax directive's parameter may be a macro usage, since a defined macro
*"can be used anywhere"* (1364-2005 §19.3.1, 1800-2017 / 1800-2023 §22.5.1): its value is then
`` {kind: "macro_usage", name: [[trivia], "`M"], arguments: "(…)" | []} `` in place of the literal envelope — `` `line `N
"f.sv" 0 ``'s `number`, `` `timescale `U / `P ``'s `unit` and `precision`, `` `undef `X ``'s `name`,
`` `default_nettype `` / `` `unconnected_drive `` / `` `begin_keywords `` values. A `` `timescale `` whose WHOLE
specification is one usage (`` `timescale `TS ``) is `{macro, comment}`. Under `sv_2017` / `verilog_2005` a
directive's tokens may be separated by newlines (the parameter envelopes' trivia then hold the newline pieces); under
`sv_2023` they may not. `` `default_nettype ``'s value is one of Syntax 22-7's eleven words (through `1.0.8` any
identifier passed).
A usage's `arguments` is its `( … )` span as one string, LEXED per §22.5.1 (*"Actual arguments and defaults shall not
contain comma or right parenthesis characters outside matched pairs of left and right parentheses (), square brackets
[], braces {}, double quotes "", or an escaped identifier"*): a string, a block comment and an escaped identifier are
atoms, so `` `include `M(")") `` and `` `M(\a) ) `` close at the last `)`; a string left open refuses the directive
(`SVPP-0015`, release `1.0.9`).
### Verified surface totals

The full typed surface of contract `1.0.8` is **100 return annotations
across 46 distinct rules** — **94 `return_object` + 4 `return_scalar` +
2 `return_array`** (the arrays: `macro_formals`, `pragma_expression_list`;
the scalars: `condition_text -> $text`, `macro_default_group_atom -> $1`,
`macro_usage_arguments -> $text`, `pragma_value_list -> $2`), AST-dump
schema version `6`, parser release `1.0.8`. These numbers are transcribed
from `generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 100`; 46 distinct `rule` values) and its embedded copy
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
The history: 64 / 27 at SVPP-Slice-1; 66 / 28 after `1.0.2`'s
`pp_if_keyword`; 67 / 29 at `1.0.5`; 68 / 31 at `1.0.7`; 100 / 46 at
`1.0.8`. The machine-checkable enumeration of every
`(rule, branch_index, annotation_type, normalized_text)` tuple is those
two artifacts; this section is curated; if it and either artifact
disagree, the artifact wins, and this integration contract wins over the
per-family mdBook.

### Resolved defect — `SVPP-0001` (fixed in 1.0.2)

The released `1.0.1` parser shipped one accepted shape defect,
`SVPP-0001`, **fixed in parser release `1.0.2` / schema `2`** and
tracked (status `Released`) in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`SVPP-0001`) and at
the head of this contract under
[Resolved Defects — `SVPP-0001`](#resolved-defects--svpp-0001-fixed-in-release-102-schema-2).
*Historical (release `1.0.1`, schema `1`):* `pp_if_branch.keyword`
emitted `"<invalid_sequence_access>"` for `` `ifdef`` / `` `ifndef``
conditional input. Root cause:
`pp_if_branch := (kw_ifdef | kw_ifndef) macro_name … -> {keyword: $1,
…}` bound `$1` to an **inline alternation group** — the same
inline-alternation-`$N` emit-time defect class fixed for `rtl_const_expr`
in RTL-CE-Slice-2 (and tracked for `rtl_frontend` / `vhdl` `binop_chain`).
The `` `define`` / non-conditional surface was unaffected. **Fix
(proven RTL-CE-Slice-2 playbook):** the inline alternation is lifted into
the named rule `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef
-> {kind: "ifndef"}`; `pp_if_branch`'s annotation is unchanged
(`{keyword: $1, …}`) but `$1` now binds the clean named rule. The
verified post-fix output is clean — for
`` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n`` the
`if_branch.keyword` is now `{kind: "ifdef"}` (or `{kind: "ifndef"}` for
an `` `ifndef`` guard), with **no** `<invalid_sequence_access>`
anywhere. Consumers read the conditional polarity directly from
`if_branch.keyword.kind`; the guard macro stays at the outer
`if_branch.macro`. The honest pre-fix history is retained in the
[Schema Versioning](#schema-versioning) schema `1.0.0` row.

## Conditional Compilation and Macro Body Fragments

This section is the consumer-facing walk contract for the two deep
substructures the [`pp_item` dispatch](#the-10-branch-pp_item-dispatch)
hands back: the **conditional-compilation tree** (reached via the
`pp_item` `"conditional"` branch → `pp_conditional`) and the
**macro-body / macro-default fragment streams** (reached via the
`pp_item` `"define"` branch → `pp_define` → `macro_body` and, for
parameterised macros, `macro_formal` → `macro_default_value`). Every
shape, `kind`, field name, branch count, and grammar line reference
below is transcribed from the live inventory
`generated/systemverilog_preprocessor_return_annotations.json`
(`version: 1`, `grammar: "systemverilog_preprocessor"`,
`annotation_count: 66`, **28 distinct rules**) and verified against
`grammars/systemverilog_preprocessor.ebnf`. The inventory is the ground
truth: where this prose and the inventory disagree, the inventory wins.

### The conditional-compilation tree

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

### The `macro_body` fragment kinds

The `pp_item` `"define"` branch carries a `body` that is the typed
`pp_define` (`{name, formals, body}`, line 33). `pp_define.body` is, for
a non-bodyless macro, the typed `macro_body` object; for a bodyless
`` `define`` it is `[]` (the `macro_body?` optional). Per
`grammars/systemverilog_preprocessor.ebnf` (lines 112–124):

```ebnf
macro_body          := macro_body_fragment+
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

Release `1.0.9` adds the twelfth kind, `` {kind: "directive", body: [[trivia], "`ifdef"]} `` — a compiler directive inside
macro text, processed where the macro is used (through `1.0.8` a `macro_reference`) — and gives the fragments macro
text's own layout (a block comment crosses a line only where its newline is backslash-preceded under `sv_2017` /
`verilog_2005`, anywhere under `sv_2023`). The punctuation fragments are regex terminals (`SVPP-0014`).

`macro_body` is a single-branch `return_object`
(`{fragments: $1}`, line 112): `fragments` is a non-empty array (the
`macro_body_fragment+`) of typed `macro_body_fragment` objects in
source order. `macro_body_fragment` is a `kind`-tagged dispatcher. The
inventory confirms exactly **12** `macro_body_fragment` branches
(`branch_index` 0–11, every one `annotation_type: "return_object"`; 9 through `1.0.7`, 11 at `1.0.8`):

| Branch | `kind` | Fields | Captured source text |
|---|---|---|---|
| 0 | `"line_continuation"` | _(none)_ | A backslash immediately before the newline: the macro text continues on the next line (§22.5.1; `1.0.8`, `SVPP-0006`). A backslash followed by white space is not a continuation. |
| 1 | `"token_paste"` | _(none)_ | The ``` `` ``` token-paste operator |
| 2 | `"stringize"` | _(none)_ | The `` `" `` stringize operator |
| 3 | `"escaped_quote"` | _(none)_ | The `` `\`" `` escape (§22.5.1; `1.0.8`, `SVPP-0009`) |
| 4 | `"macro_reference"` | `body` | A nested macro reference `` `IDENT`` (`macro_reference := bt_identifier`) |
| 5 | `"text"` | `body` | A run of literal body text. Since `1.0.8` a block comment, a `//` comment (up to a continuation backslash) and a plain string literal are each consumed WHOLE inside the run (§22.5.1: *"Macro substitution and argument substitution shall not occur within string literals"*; a comment *"shall not become part of the substituted text"*), and a backslash only with the character after it |
| 6 | `"lparen"` | _(none)_ | A literal `(` outside a string or comment |
| 7 | `"rparen"` | _(none)_ | A literal `)` outside a string or comment |
| 8 | `"comma"` | _(none)_ | A literal `,` outside a string or comment |
| 9 | `"question"` | _(none)_ | A literal `?` outside a string or comment |
| 10 | `"colon"` | _(none)_ | A literal `:` outside a string or comment |

The structural punctuation kinds are emitted as discrete fragments rather than
folded into `"text"` so a consumer can reconstruct macro-argument
parentheses and `?:` ternary structure inside the un-expanded body
without re-lexing; since `1.0.8` that punctuation is never taken from inside a
string literal or a comment. Only `"macro_reference"` and `"text"` carry a
`body`; the other nine kinds are bare `{kind}` objects.

### The `macro_default_atom` kinds

For a parameterised macro, `pp_define.formals` is the typed
`macro_formals` — a clean `[macro_formal, …]` array (line 110, the
`1.0.3` POST-SV-AUDIT Category-A extraction-spread `[$2, $3::2*]`; at
≤ release `1.0.2` / schema `2` it was the raw `{first, rest}` iteration
envelope — see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3));
each `macro_formal` (`{name, default}`, line 112) may carry a `default`
that is the typed `macro_default_value` object. Per
`grammars/systemverilog_preprocessor.ebnf` (lines 99–110):

```ebnf
macro_default_value := macro_default_atom+
                    -> {atoms: $1}
macro_default_atom  := macro_token_paste  -> {kind: "token_paste"}
                     | macro_stringize    -> {kind: "stringize"}
                     | macro_reference    -> {kind: "macro_reference", body: $1}
                     | macro_default_text -> {kind: "text",  body: $1}
                     | lparen             -> {kind: "lparen"}
                     | rparen             -> {kind: "rparen"}
                     | question           -> {kind: "question"}
                     | colon              -> {kind: "colon"}
```

`macro_default_value` is a single-branch `return_object`
(`{atoms: $1}`, line 99): `atoms` is a non-empty array (the
`macro_default_atom+`) of typed `macro_default_atom` objects in source
order. `macro_default_atom` is a `kind`-tagged dispatcher. The
inventory confirms exactly **8** `macro_default_atom` branches
(`branch_index` 0–7, every one `annotation_type: "return_object"`) —
this **matches the expected 8**. Note this is one fewer than
`macro_body_fragment`: `macro_default_atom` has **no `"comma"` kind**
(a comma terminates a default-argument value in the `macro_formals`
`lparen macro_formal (comma macro_formal)* rparen` grammar, so it is
not part of an atom stream), and its text leaf is `macro_default_text`
(line 110, char class `/[^`(),?:\r\n]+/`) rather than
`macro_body_text`:

| Branch | `kind` | Fields | Captured source text | Grammar line |
|---|---|---|---|---|
| 0 | `"token_paste"` | _(none — bare `{kind}`)_ | The `` `` `` token-paste operator | line 101 |
| 1 | `"stringize"` | _(none — bare `{kind}`)_ | The `` `" `` stringize operator | line 102 |
| 2 | `"macro_reference"` | `body` | A nested macro reference `` `IDENT``; `body` is the referenced macro envelope | line 103 |
| 3 | `"text"` | `body` | A run of literal default text (`macro_default_text := inline_trivia /[^`(),?:\r\n]+/`); `body` is the captured text envelope | line 104 |
| 4 | `"lparen"` | _(none — bare `{kind}`)_ | A literal `(` | line 105 |
| 5 | `"rparen"` | _(none — bare `{kind}`)_ | A literal `)` | line 106 |
| 6 | `"question"` | _(none — bare `{kind}`)_ | A literal `?` | line 107 |
| 7 | `"colon"` | _(none — bare `{kind}`)_ | A literal `:` | line 108 |

**Consumer guidance — composing default-argument atoms.** A
default-argument value is the `atoms` array of a `macro_default_value`,
reached via `pp_define.formals` (the clean `macro_formal[]` list as of
`1.0.3` / schema `3` — at ≤ `1.0.2` / schema `2` this was the raw
`macro_formals.first` / `macro_formals.rest[]` `{first, rest}` envelope;
see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3))
→ each `macro_formal` → `macro_formal.default`
(`[]` when the formal has no `= default`). Concatenate the atoms in
order, dispatching each on `kind`: `"text"` / `"macro_reference"`
contribute their `body`; `"lparen"` / `"rparen"` / `"question"` /
`"colon"` contribute their literal punctuation (these let a consumer
reassemble a parenthesised or ternary default expression); `"token_paste"` /
`"stringize"` contribute the `` `` `` / `` `" `` operators. There is no
`"comma"` atom by construction — a comma always closes the current
default and begins the next `macro_formal`.

### Consumer guidance — walking a `pp_define` body

To walk a `` `define`` end to end: dispatch a `pp_item` on
`kind == "define"` and take `body` — the typed `pp_define`
(`{name, formals, body}`, line 33). `name` is the macro identifier
envelope. `formals` is `[]` for an object-like macro, else the typed
`macro_formals` — a clean flat `macro_formal[]` array (the `1.0.3` /
schema `3` POST-SV-AUDIT Category-A `[$2, $3::2*]` extraction-spread; at
≤ `1.0.2` / schema `2` this was the raw `{first, rest}` envelope where a
consumer had to walk `formals.first` then `formals.rest[][1]` past the
`comma` separator — see
[AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--macro_formals-category-a-raw-envelope--clean-list-schema-2--3)).
Iterate `pp_define.formals` directly (each element a typed
`macro_formal` `{name, default}`), recursing into
`macro_default_value.atoms` (the `macro_default_atom` stream above) for
any non-`[]` `default`. `pp_define.body` is `[]` for a bodyless macro,
else the typed `macro_body` (`{fragments}`); iterate `fragments`,
dispatching each typed `macro_body_fragment` on its `type`-equivalent
`kind` tag (`"text"` / `"macro_reference"` / `"token_paste"` /
`"stringize"` / `"lparen"` / `"rparen"` / `"comma"` / `"question"` /
`"colon"`) per the table above to reconstruct the un-expanded macro
body in source order. The only `type`-discriminated object on this
whole path is the `systemverilog_preprocessor_file` root; `pp_item`,
`macro_body_fragment`, and `macro_default_atom` all dispatch on `kind`.
This contract documents the surface as it exists in the inventory; the
schema axis is the AST-dump schema version **`3`** (see
[Schema Versioning](#schema-versioning)) and the parser release is
**`1.0.3`**, with **66 return annotations across 28 distinct rules**
(65 `return_object` + 1 `return_array`) — the count is unchanged by the
`1.0.3` POST-SV-AUDIT correction (`macro_formals` is still one rule /
one annotation; only its annotation form changed `return_object` →
`return_array`, `{first, rest}` → `[$2, $3::2*]`). The `1.0.2`
`SVPP-0001` fix touched only the conditional tree's
`pp_if_branch.keyword` / new `pp_if_keyword` rule.
Where this prose and the inventory disagree, the inventory wins.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog_preprocessor.ebnf`
- Runtime execution stage (`ENGINE-UNIVERSAL-SERVICES.113`, 2026-09-29: the SystemVerilog family's own component, behind the engine's neutral `rust/src/preprocessing_stage.rs`):
  - `rust/src/families/systemverilog/preprocessor.rs`
- Generated-parser build discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH`
- Current operational guide:
  - `PGEN_USER_GUIDE.md`
- Live status surface:
  - `docs/book/src/roadmap-and-live-status.md`

## Stable Integration Surface
- Current downstream-facing contract is narrower than the main SystemVerilog/VHDL/regex host surface.
- The repository does expose generated-parser registry coverage for `systemverilog_preprocessor`, but it does not currently publish a dedicated general-purpose embedding API profile for it in `pgen::embedding_api`.
- The practical stable surface today is:
  - the Rust preprocessor execution/runtime module in `rust/src/families/systemverilog/preprocessor.rs` (reached by the CLIs through `pgen::preprocessing_stage::stage_for("systemverilog")`)
  - the executable quality and differential gates documented in `PGEN_USER_GUIDE.md`

## Build / Availability Requirements
- Do not treat internal parser-registry exposure as equivalent to a published general-purpose downstream host contract.
- If a downstream project needs a generic public embedding API for `systemverilog_preprocessor`, that should be treated as new product-surface work, not assumed from current internal registry availability.

## Validation / Release Gates
- `python3 docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py --bank --probe rust/target/release/parseability_probe` — the reproducer bank (119 cases, ACCEPT and REJECT, each with its clause and, for 19, the profile it is read under); needs no corpus.
- the same census under each edition: `--profile sv_2023`, and `--profile verilog_2005` over the v2005 lane's own population — each with its own disposition table, `--check-dispositions --profile P`; every reading prints `SVPP-CORPUS-CENSUS-ERROR-TESTS:` and refuses an accepted suite error test without a written reason.
- `python3 docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py --probe rust/target/release/parseability_probe --before <the previous release's probe>` then `--check-dispositions` — the generated parser over the 16,336-file SystemVerilog corpus, transitions and AST identity against the previous release, every rejection adjudicated (run at every release of this family; it reads the SV raw lane's dump).
- `make -C rust SHELL=/bin/bash sv_preprocessor_quality_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_curated_differential_gate`
- `make -C rust SHELL=/bin/bash sv_preprocessor_template_differential_gate`

## Runtime Execution Stage — the EXPANSION (`SVPP-EXPANSION`, 2026-09-09; the directive parser's grammar, release and schema unchanged)
- The generated `systemverilog_preprocessor` parser this contract describes reads DIRECTIVES into an AST. The
  EXPANSION — executing those directives on a file so the `systemverilog` parser reads IEEE 1800-2017 text with
  clause 22 already applied — is a **sibling deliverable** in the runtime module `rust/src/families/systemverilog/preprocessor.rs`,
  governed by the `SVPP-EXPANSION` tree and contracted in the SV integration contract's *"Front End — the
  preprocess→parse pipeline"* section (`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`),
  where the command, the four-part context, the source-map rule, the refusal codes and the library live.
- Behaviour a consumer of the runtime module relies on, landed 2026-09-09 (each rule with its clause and a
  corpus reproducer in `docs/tasks/SVPP-EXPANSION.md` `.5`, `.6b.2`, `.6b.5`): a missing or extra macro actual is
  REFUSED (`E_SVPP_MISSING_MACRO_ARGUMENT` / `E_SVPP_EXTRA_MACRO_ARGUMENT`), recursion is REFUSED by provenance
  (`E_SVPP_RECURSIVE_MACRO`), `//` comments are stripped from macro text, a well-formed `` `line `` is consumed,
  a `` `pragma protect `` envelope is consumed (`E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE` when open at EOF), block
  comments span lines, a `$ident` is never substituted, the fifteen `` `SV_COV_* `` macros are predefined
  (§40.3.1), `--unit-file` (`--sv-unit-file` through `1.0.247`) folds a predecessor file's macro table into the unit, and the event log gained the
  `LineDirective` and `ProtectedEnvelope` kinds. The single-command chain is
  `parseability_probe --parse systemverilog FILE --profile P --preprocess …` (exit 3 = the preprocessor refused).
- **`` `begin_keywords `` regions (IEEE 1800-2017 §22.14), landed 2026-09-14 (`SVPP-EXPANSION.8`).** The
  expansion stage applies the clause as TEXT: inside a region, every token the PARSE PROFILE reserves and
  the region's `version_specifier` does not is re-emitted as an ESCAPED identifier (`logic` → `\logic `),
  which §5.6.2 makes not a keyword and §5.6.1 makes the same identifier as the unescaped spelling. The
  profile is `SvPreprocessorConfig::parse_profile` — `--profile` on both commands (`parseability_probe`
  and, since `ENGINE-UNIVERSAL-SERVICES.113`, `ast_pipeline --preprocess systemverilog`; `--sv-keyword-profile`
  accepted through `1.0.247`); `None` is the parser's unprofiled reading, which reserves IEEE 1800-2017
  Table B.1. Regions nest and are UNIT state (they carry across `--unit-file` boundaries, per the clause).
  The per-version tables are `rust/src/families/systemverilog/keyword_versions.rs` (the family's own component),
  derived from the in-repo LRM text and held equal to it by the `SV-KEYWORD-VERSIONS` doctrine. New
  diagnostics: `E_SVPP_UNKNOWN_KEYWORDS_VERSION` (§22.14's *"It shall be an error if an implementation does
  not recognize the version_specifier"*), `W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE` (the region reserves more
  than the profile — text cannot create a keyword, so the region is applied only in the direction it can),
  `W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION`, `W_SVPP_UNMATCHED_END_KEYWORDS`,
  `W_SVPP_MALFORMED_KEYWORDS_DIRECTIVE`; new event kind `KeywordsRegion`. ⛔ A consumer walking the AST sees
  the ESCAPED spelling at a rendered site — `{kind: "escaped_identifier"}`-shaped, body `\logic ` — and the
  source map maps it back to the unescaped bytes the author wrote.
- **The order around a ``` `` ``` paste, landed 2026-09-15 (`SVPP-EXPANSION.5`(i)).** A paste constructs an
  identifier (§22.5.1), and the identifier is often a macro name: an operand that is a DEFINED macro usage is
  expanded before the join (`` `A``b```C `` → `abc`), an UNDEFINED usage joined by a paste is looked up as the
  whole constructed name in place (`` `CHOICE_``FOO```SUFFIX `` → `` `CHOICE_FOO_MAGIC ``, `` `QA``_b `` →
  `` `QA_b ``), and a constructed function-like usage (`` `uvm_pack_``TYPE``(ARG, p) ``) collects its arguments
  as written. These are Verilator 5.050's and sv2v's readings, measured against both with a re-runnable
  three-way probe; Icarus 13.0 reads the left side as soon as it is joined and is recorded as the outlier. The
  paste tokens now survive substitution and are consumed by the rescan; inside any string literal they are
  removed at substitution as before.
- **The `` `"…`" `` string region, landed 2026-09-15 (`SVPP-EXPANSION.5`(j)).** §22.5.1's three things — *"the
  quotation mark, substitution of actual arguments, and expansions of embedded macros"* — are done by ONE renderer in
  the rescan, for the object-macro path (never rendered before: OpenTitan `otbn_ssctrl_if.sv`'s
  `` `define SSCTRL_PATH `"`SSCTRL_HPATH`" `` reached the parser with its markers on) and the function-macro path
  (whose substitution now emits the region with the actuals in place); an embedded usage is expanded, `` `\`" `` is
  `\"` (the clause's own `msg` example was mis-rendered), an undefined or argument-less usage stays as written. A region
  in a SOURCE line is not rendered (the form is defined for macro text; Verilator renders it, Icarus does not — recorded).
  With it, `()` is one empty actual when the macro declares formals (`.5`(j′), §22.5.1 *"empty … by nothing if no
  default"*), the reading both reference tools take.
- **A plain `"…"` string in macro text is opaque, landed 2026-09-15 (`SVPP-EXPANSION.5`(k)).** §22.5.1 `:370-372`: the
  characters of a plain string literal "become part of the string in the replaced macro text", its own example
  `` `define H(x) "Hello, x" `` printing `Hello, x`. `SvPreprocessorConfig::plain_string_substitution` /
  `--stage-opt plain_string_substitution=…` (`--sv-plain-string-substitution` through `1.0.247`) names the two measured deviations — `paste_delimited` (only a formal marked by paste
  tokens on both sides is substituted: OpenTitan's `"…[``i``]…"`) and `all` (every formal, every paste: Icarus 13.0) —
  with `standard` the default. The engine had read `all` since the runtime module's first commit, which mangled 66 of
  the corpus's 72 in-string formal sites (a message's prose replaced by an actual) to serve 6.
- The live measurement of the expansion is the SV parser book's chapter
  `docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md`, not this family's status page: this
  family's corpus leg stays NOT MEASURED because no corpus of directive files is wired, and the expansion's
  corpus is the SystemVerilog family's.

## Known Gaps — release 1.0.9

Open in this parser, owned by `SV-CORPUS-GRAD.13c.2v.17` and ordered fixed — defects, not a narrowed promise:

- a macro usage whose actuals continue onto the next line INSIDE a directive parameter (`` `timescale `T(1ns,⏎1ps) ``)
  is read as a usage without actuals followed by text, under every profile; whether the `(` opens actuals depends on
  the macro's definition, which this parser does not have, so the reading is that leaf's first deliverable;
- under `sv_2023`, a block comment crossing a line inside such actuals is accepted the same way (the text after the
  usage is read as the directive's following text).

The PIPELINE's expansion stage carries seven edition and lexical findings made
while designing this release — `SVPP-EXPANSION.5`(w) (`` `line 0 `` consumed), (x) (an ignored group not lexed), (y) (a
parenthesized conditional outside 1800-2023), (z) (1800 macro features under `verilog_2005`), (aa) (the line discipline
read per directive, not per edition), (ab) (an empty formal list accepted), (ac) (a macro usage's actuals closed at a
`)` inside `[ ]` / `{ }` or an escaped identifier) — each ordered fixed in the stage's own release next, which measures
the SystemVerilog pipeline movers they cause.

## Scope / Non-Goals
- The EXPANSION stage is contracted in the SV integration contract (section above); this document contracts the directive parser's AST.
- A directive with a defined end may be followed by text on its line, and since `1.0.9` a directive may be PRECEDED by text on its line (IEEE 1800-2023 §22.2: *"A language element is allowed on the same line before a directive"*); the tree records such text as a `non_directive_line` item that ends where the directive begins. `` `include `` and `` `line `` own their line (§22.4, §22.12); `` `pragma `` and a `` `define ``'s text run to the line end.
- This parser does not decide LIVENESS (which conditional groups a compilation compiles) — that needs the compilation's symbols and includes, and is the expansion stage's. It records every group's structure, and a line inside a group that does not parse as a `malformed_directive` item.
- This document is intentionally explicit that `systemverilog_preprocessor` does not yet have the same published host-embedding shape as `systemverilog`, `vhdl`, or `regex`.
- Downstream consumers should not couple themselves to internal generated parser modules as if they were already a stable public API.
- If a downstream integrator still reports a reproducible preprocessor/runtime bug, use `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md` and log accepted released-parser issues in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Companion Documentation — sv_preprocessor Parser Integration mdBook

This contract is the **downstream integration surface**: the host-API
envelope, the `pp_item` dispatch / conditional-compilation / macro-body
shapes a consumer compiles against, and the release/schema axes. It does
not duplicate the per-rule walkthroughs or worked examples — those live
in the companion artifacts below. Each surface is authoritative for a
different thing; consult the matching one and respect the precedence
order stated at the end of this section.

| Surface | Path | Authoritative for |
|---|---|---|
| **This contract** | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | The downstream integration surface: AST-dump envelope, `systemverilog_preprocessor_file` root, the 10-branch `pp_item` dispatch, the conditional-compilation tree, and the macro-body / macro-default fragment streams. See [AST Envelope and pp_item Dispatch](#ast-envelope-and-pp_item-dispatch) and [Conditional Compilation and Macro Body Fragments](#conditional-compilation-and-macro-body-fragments). |
| **Per-parser mdBook** | `docs/systemverilog_preprocessor_parser_book/` (source `src/*.md`; tracked HTML at `docs/systemverilog_preprocessor_parser_book-html/`) | The per-rule reference and teaching surface: build recipe, public API, AST-envelope walkthrough, every rule shape, per-feature worked examples (including the conditional worked example with the `SVPP-0001` schema-`1`→`2` fix transition), schema-versioning timeline, glossary, changelog index. Curated, not machine-checked. Listed in `README.md` § "Per-Parser Integration Reference Books". |
| **Shape-contract manifest** | `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json` | The machine-checkable shape lock embedded in the regression test. Content-identical to the live inventory on the `(rule, branch_index, annotation_type, normalized_text)` tuples (the embedded copy omits only the diagnostic `raw_text` field). Drift fails the AST-shape-contract test. |
| **Declared-annotation inventory** | `generated/systemverilog_preprocessor_return_annotations.json` | The live machine-checkable enumeration of every typed-shape annotation the systemverilog_preprocessor grammar emits (`version: 1`, `grammar: "systemverilog_preprocessor"`, `annotation_count: 66`, **28 distinct rules**; 65 `return_object` + 1 `return_array` — the `return_array` is `macro_formals`, the `1.0.3` POST-SV-AUDIT Category-A correction). The generator-side source of truth for the typed surface. |
| **Embedding-API contract** | `rust/docs/EMBEDDING_API_CONTRACT.md` | The canonical host-API truth: the `AstDumpPayload` struct (`dump_json` / `truncated` / `full_bytes` / `emitted_bytes`), the entry-point signatures, the truncation diagnostic envelope, and the stable diagnostics. The struct shape this contract documents is transcribed from there. |
| **Released-parser bug ledger** | `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | The accepted-bug log for the released systemverilog_preprocessor parser; the `SVPP-0001` defect lives there (status `Released`, fixed in parser release `1.0.2` / schema `2`). Consult before integrating around a suspected parser defect; file new accepted bugs here per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`. |

Precedence when surfaces disagree (highest first): the **embedding-API
contract** (`rust/docs/EMBEDDING_API_CONTRACT.md`) wins for the host-API /
`AstDumpPayload` truth; the **declared-annotation inventory**
(`generated/systemverilog_preprocessor_return_annotations.json`) and its
embedded shape-contract manifest copy
(`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`)
win for the exact typed-shape enumeration; **this integration contract**
wins over the **per-parser mdBook** for downstream compliance. Report any
disagreement as a documentation bug rather than silently coding to the
lower-precedence surface.

### Gate Recipe

The exact, copy-pasteable per-family commands a downstream integrator or
releaser runs. Each is verified against the repo (`rust/Makefile`,
`docs/systemverilog_preprocessor_parser_book/src/build-recipe.md`,
`rust/src/ast_shape_contract.rs`); none are invented — do not substitute
flags.

**1. On-demand parser regen.** The systemverilog_preprocessor parser is
on-demand-only (not in the default `cargo test --features
generated_parsers` build). Build `ast_pipeline`, then regenerate the
parser from `grammars/systemverilog_preprocessor.ebnf` (run from
`rust/`, per
`docs/systemverilog_preprocessor_parser_book/src/build-recipe.md`
§ "Cold-clone build"):

```bash
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/systemverilog_preprocessor.ebnf \
    --generate-parser --output ../generated/systemverilog_preprocessor_parser.rs
```

To wire the regenerated parser into a cargo build, point
`PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH` at the absolute path of the
generated file before `cargo build --release --features
generated_parsers` (see
`docs/systemverilog_preprocessor_parser_book/src/build-recipe.md`
§ "Wiring into a downstream Cargo build").

**2. Per-family book gate.** Builds the sv_preprocessor parser book and
verifies the tracked HTML landing pages (Makefile target
`systemverilog_preprocessor_parser_book_gate`, `rust/Makefile`
line 751, which runs
`rust/scripts/systemverilog_preprocessor_parser_book_gate.sh`):

```bash
make -C rust SHELL=/opt/homebrew/bin/bash systemverilog_preprocessor_parser_book_gate
```

> Shell note: this contract (and the book's `build-recipe.md`)
> deliberately use the Homebrew bash-4+ path
> `SHELL=/opt/homebrew/bin/bash`, consistent with the other per-family
> contracts. `README.md` § "Per-Parser Integration Reference Books"
> still prints the `SHELL=/bin/bash systemverilog_preprocessor_parser_book_gate`
> form; that README-wide discrepancy is tracked as
> `DOC-README-SHELL-0001` — prefer the Homebrew path used here.

**3. AST-shape-contract regression lock.** With the generated backend
wired in (`PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH` exported), run
the shape-contract test that diffs the running generated parser against
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
(test fn
`systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser`
in the `pgen::ast_shape_contract` library module,
`rust/src/ast_shape_contract.rs` line 817):

```bash
cargo test --lib --features generated_parsers systemverilog_preprocessor_ast_shape_contract
```

The substring `systemverilog_preprocessor_ast_shape_contract` selects
exactly the
`systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser`
test (it does not match the `vhdl_*` / `rtl_*` shape-contract tests in
the same module). Any drift between the running parser's emitted shapes
and the locked manifest fails this test, surfacing the change before
release. (The `1.0.2` `SVPP-0001` fix is locked here — the corrected
`pp_if_branch.keyword` / new `pp_if_keyword` shape is part of the locked
66-entry baseline; any reversion to the pre-fix
`<invalid_sequence_access>` shape fails this test immediately. The
schema was bumped `1 → 2` in same-commit lockstep with this fix.)

**4. Validation / release gates.** Anyone publishing a parser-release
version bump also runs the per-family gates enumerated in
[Validation / Release Gates](#validation--release-gates) (the
`sv_preprocessor_quality_gate`, `sv_preprocessor_curated_differential_gate`,
and `sv_preprocessor_template_differential_gate` targets). That section
is the canonical list; it is not repeated here.

## Glossary

Contract-scoped definitions of the terms a downstream integrator needs to
read this document. Where a term has a normative definition, this
contract is authoritative; the per-parser book's
[glossary](../systemverilog_preprocessor_parser_book/src/glossary.md)
paraphrases the same terms for quick lookup. Numbers below are pinned to
contract `1.0.3` / AST-dump schema `3` / parser release `1.0.3` /
**66 return annotations across 28 distinct rules** (65 `return_object`
+ 1 `return_array` — the `return_array` is `macro_formals`, the `1.0.3`
POST-SV-AUDIT Category-A correction).

- **`AstDumpPayload`** — the success return of the
  systemverilog_preprocessor AST-dump host entry points (defined in
  `rust/src/embedding_api.rs`, contract in
  `rust/docs/EMBEDDING_API_CONTRACT.md`). A canonical-JSON payload string
  plus truncation metadata, with **exactly four fields**: `dump_json`,
  `truncated`, `full_bytes`, `emitted_bytes`. It does **not** carry
  `root` / `schema_version` / `grammar` / `profile` members — see
  [The `AstDumpPayload` envelope](#the-astdumppayload-envelope) for the
  precise accuracy note.
- **`dump_json`** — the `AstDumpPayload` field holding the canonical
  (key-sorted) JSON encoding of the typed systemverilog_preprocessor
  AST. Parse this string to obtain the
  `systemverilog_preprocessor_file` root object. When `truncated` is
  `true` this string is replaced by the truncation diagnostic envelope,
  not the AST.
- **Truncation diagnostic envelope** — the deterministic JSON object
  that replaces the AST in `dump_json` when `max_ast_bytes` is exceeded.
  It carries `pgen_dump_contract_version` (currently `1`), `kind:
  "pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
  "parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`.
  Consumers must check `truncated` (or detect `kind ==
  "pgen_ast_dump_truncation"`) before treating `dump_json` as a
  systemverilog_preprocessor AST. `pgen_dump_contract_version` is a
  member of **this envelope only**, never of `AstDumpPayload` itself.
- **AST-dump schema version** — the integer version axis tracking the
  AST output shape, currently `3`, pinned by this contract (see
  [Schema Versioning](#schema-versioning)). It is **not** a field of
  `AstDumpPayload`; it is the contract-tracked axis. Bumped only when
  the emitted shape changes in a way consumers may need to adapt to (new
  annotation on a previously-unannotated rule, restructured annotation,
  user-visible grammar-shape change). It was bumped `1 → 2` by the
  `1.0.2` `SVPP-0001` correctness fix because `pp_if_branch.keyword`
  changed shape in a consumer-visible way (was the malformed
  `<invalid_sequence_access>` object at schema `1`, now the typed
  `{kind: "ifdef"|"ifndef"}` `pp_if_keyword` polarity discriminator),
  and bumped `2 → 3` by the `1.0.3` POST-SV-AUDIT Category-A correction
  because `pp_define.formals` (`macro_formals`) changed from the raw
  `{first, rest}` iteration envelope to the clean `[macro_formal, …]`
  list (`PGEN-POST-SV-AUDIT-0002`; a deliberate audit-driven shape
  correction, **not** a released-parser bug).
- **Parser release version** — the parser library's release identity,
  currently `1.0.3`. Bumped on every functional change (bug fixes, perf
  work, grammar changes). Moves independently of the schema version; the
  `1.0.3` release carries AST-dump schema `3`.
- **`pp_item` dispatch** — the primary top-level dispatcher: a
  **10-branch** `kind`-tagged shape
  (`grammars/systemverilog_preprocessor.ebnf` lines 18–27). Every parse
  roots at `{type: "systemverilog_preprocessor_file", items: [...]}`;
  each element of `items` is one typed `pp_item` object in source order.
  The `systemverilog_preprocessor_file` root is the only `type`-tagged
  dispatch object; `pp_item`, `condition_atom`, `macro_default_atom`,
  and `macro_body_fragment` all dispatch on `kind`. See
  [The 18-branch `pp_item` dispatch](#the-18-branch-pp_item-dispatch).
- **The 18 `pp_item` kinds** — `"define"`, `"undef"`, `"include"`,
  `"timescale"`, `"default_nettype"`, `"celldefine"`,
  `"endcelldefine"`, `"resetall"`, `"undefineall"`,
  `"unconnected_drive"`, `"nounconnected_drive"`, `"pragma"`, `"line"`,
  `"begin_keywords"`, `"end_keywords"`, `"conditional"`,
  `"non_directive_line"`, `"blank_line"` (branch indices 0–17; the eight
  clause-22 kinds since `1.0.8`). Every kind except the three
  bodyless ones (`"celldefine"`, `"endcelldefine"`, `"blank_line"`)
  carries a `body` holding the underlying typed shape.
- **The 7 directive shapes** — the seven non-conditional directive
  rules below the `pp_item` dispatch, each a single-branch
  `return_object`: `pp_define` (`{name, formals, body}`), `pp_undef`
  (`{name, comment}`), `pp_include` (`{path, comment}`), `pp_timescale`
  (`{unit, precision, comment}`), `pp_default_nettype`
  (`{nettype, comment}`), `pp_celldefine` (`{comment}`),
  `pp_endcelldefine` (`{comment}`). See [Per-directive shapes](#per-directive-shapes).
- **Conditional-compilation tree** — the typed tree the
  `pp_item` `"conditional"` branch hands back: `pp_conditional`
  (`{if_branch, elsif_branches, else_branch}`) over `pp_if_branch`
  (`{keyword, macro, tail, items}` — where `keyword` is the typed
  `pp_if_keyword` polarity object `{kind: "ifdef"|"ifndef"}` as of the
  `1.0.2` `SVPP-0001` fix), `pp_elsif_branch` (`{condition, items}`),
  `pp_else_branch` (`{tail, items}`), and `pp_endif` (`{tail}`), plus
  the 2-branch `pp_if_keyword` dispatch rule
  (`{kind: "ifdef"}` / `{kind: "ifndef"}`) the `1.0.2` fix added. Each
  branch's `items` is a nested `pp_item*` array; the closing `pp_endif`
  is consumed positionally and is **not** re-emitted as a
  `pp_conditional` field. See
  [The conditional-compilation tree](#the-conditional-compilation-tree).
- **`macro_body_fragment` / `macro_default_atom`** — the two
  `kind`-tagged fragment-stream dispatchers. `macro_body` is
  `{fragments}` over **9** `macro_body_fragment` kinds (`"token_paste"`,
  `"stringize"`, `"macro_reference"`, `"text"`, `"lparen"`, `"rparen"`,
  `"comma"`, `"question"`, `"colon"`); `macro_default_value` is
  `{atoms}` over **8** `macro_default_atom` kinds (the same set **minus
  `"comma"`**, whose text leaf is `macro_default_text`). Only the
  `"macro_reference"` / `"text"` kinds carry a `body`; the others are
  bare `{kind}`. See [The `macro_body` fragment kinds](#the-macro_body-fragment-kinds)
  and [The `macro_default_atom` kinds](#the-macro_default_atom-kinds).
- **Recursive envelope** — the default JSON shape produced by
  un-annotated rules: a recursive composition of arrays (sequences,
  quantified iterations such as a branch's nested `pp_item*` `items`),
  strings (terminal/regex leaves — e.g. the `macro_name` / `identifier`
  envelope, the `directive_comment_tail` / `directive_tail` tail), and
  matched-branch passthroughs (for alternations). Un-matched optionals
  are the empty array `[]`, never `null`. It is what a consumer reaches
  when descending below the typed surface (the `name` / `comment` /
  `tail` envelopes; the malformed `pp_if_branch.keyword` nested object
  of `SVPP-0001`).
- **Shape-contract manifest** — the embedded machine-checkable shape
  lock `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
  Content-identical to the declared-annotation inventory on the
  `(rule, branch_index, annotation_type, normalized_text)` tuples (omits
  only the diagnostic `raw_text` field). Drift fails the
  `systemverilog_preprocessor_ast_shape_contract_holds_against_running_generated_parser`
  regression test (see [Gate Recipe](#gate-recipe)).
- **Declared-annotation inventory** — the live machine-checkable
  enumeration of every typed-shape annotation the
  systemverilog_preprocessor grammar emits:
  `generated/systemverilog_preprocessor_return_annotations.json`
  (`version: 1`, `grammar: "systemverilog_preprocessor"`,
  `annotation_count: 66`, **28 distinct rules**; 65 `return_object` + 1
  `return_array` — the `return_array` is `macro_formals`, the `1.0.3`
  POST-SV-AUDIT Category-A correction). The generator-side source of
  truth for the typed surface; mirrored by the embedded shape-contract
  manifest copy. (The `version: 1` field is the inventory-file format
  version, distinct from the AST-dump schema version `3` and the parser
  release version `1.0.3`.)
- **Generic host AST-dump surface** — the
  `parse_grammar_profile_ast_dump*` family
  (`parse_grammar_profile_ast_dump`, the `*_result` and `*_named`
  forms). The grammar-agnostic entry points that, for the
  `systemverilog_preprocessor` grammar + `default` profile, would return
  the `AstDumpPayload`. systemverilog_preprocessor has **no**
  named-convenience entry point and (per
  [Stable Integration Surface](#stable-integration-surface)) does **not**
  currently publish a dedicated general-purpose embedding-API profile in
  `pgen::embedding_api`; the practical stable surface today is the
  runtime module `rust/src/families/systemverilog/preprocessor.rs` plus the executable
  gates. Signatures are in `rust/docs/EMBEDDING_API_CONTRACT.md`.
- **`SVPP-0001`** — the one accepted shape defect the `1.0.1` parser
  shipped, **fixed in parser release `1.0.2` / schema `2`**.
  *Historical:* `pp_if_branch.keyword` (the `$1` bound to the
  then-inline alternation `(kw_ifdef | kw_ifndef)`) surfaced, for any
  `` `ifdef``/`` `ifndef`` input, a malformed nested object containing
  three `"<invalid_sequence_access>"` strings instead of the keyword
  token — the inline-alternation-`$N` emit-time defect class also fixed
  for `rtl_const_expr` in RTL-CE-Slice-2 and tracked for
  `rtl_frontend` / `vhdl` `binop_chain`. **Fixed** by lifting
  `(kw_ifdef | kw_ifndef)` into the named rule `pp_if_keyword := kw_ifdef
  -> {kind: "ifdef"} | kw_ifndef -> {kind: "ifndef"}`
  (`grammars/systemverilog_preprocessor.ebnf` lines 71–72); `pp_if_branch`'s
  bare `keyword: $1` (line 74) now binds the clean named rule, so
  `if_branch.keyword` is the typed polarity object `{kind: "ifdef"}` /
  `{kind: "ifndef"}` with **no** `<invalid_sequence_access>` anywhere.
  Status `Released`; tracked in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` and documented at
  [Resolved Defects — `SVPP-0001`](#resolved-defects--svpp-0001-fixed-in-release-102-schema-2)
  and in the
  [Resolved defect — `SVPP-0001`](#resolved-defect--svpp-0001-fixed-in-102)
  and
  [`SVPP-0001` in the conditional-tree context](#svpp-0001-in-the-conditional-tree-context-resolved-in-102)
  notes. Consumer guidance: read the conditional polarity from
  `if_branch.keyword.kind` and the guard macro from the outer
  `if_branch.macro`. The AST-dump schema version was bumped `1 → 2` by
  this fix; consumers written against `1.0.1` must repin to schema `2`.
