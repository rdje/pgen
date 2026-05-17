# docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `rtl_frontend` parser family.

This is the document downstream projects (primarily RTLSyn) embedding the PGEN rtl_frontend parser should read first.

## Contract Identity
- Contract version:
  - `1.0.3`
- Parser release version:
  - `1.0.3`
- Embedding API contract baseline:
  - tracked under `rust/docs/EMBEDDING_API_CONTRACT.md`
- rtl_frontend AST-dump schema version:
  - `3` (POST-SV-AUDIT Category-A AST-shape corrections + the `RTL-FE-0002` `event_control_list` inline-alternation fix — see "AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT)"; the `1.0.2`/schema `2` `RTL-FE-0001` and `1.0.1`/schema `1` history is retained below)
- Last updated:
  - `2026-05-17`
- Current grammar family label:
  - `rtl_frontend`
- Per-family mdBook:
  - `docs/rtl_frontend_parser_book/` (tracked HTML at `docs/rtl_frontend_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate`
- Per-family ast-shape-contract manifest:
  - `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`

## Source Of Truth
- Grammar source:
  - `grammars/rtl_frontend.ebnf`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_RTL_FRONTEND_PARSER_PATH`
- Live closure/status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`

## Stable Integration Surface
- Grammar family:
  - `rtl_frontend`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Build / Availability Requirements
- Downstream consumers should inspect the embedding API contract for whether the rtl_frontend generated backend is available before relying on it.
- The generated backend is resolved by `rust/build.rs`, not by importing internal generated parser modules directly.

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
- Per-family book gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate`
- Per-family contract gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_generated_contract_gate`
- AST-shape contract:
  - `cargo test --lib --features generated_parsers rtl_frontend_ast_shape_contract`

## Schema Versioning

The rtl_frontend parser carries two version axes:

1. **Parser release version** (`1.0.3`). Tracks the parser library's release identity. Bumped on every functional change.
2. **AST-dump schema version** (`3`). Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 3 | 1.0.3 | **POST-SV-AUDIT Category-A AST-shape corrections + `RTL-FE-0002` `event_control_list` inline-alternation fix (consumer-visible).** The POST-SV-AUDIT.2.2 static classification pass (`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.2, tracked `PGEN-POST-SV-AUDIT-0003`) found **15 static-conclusive Category-A raw-envelope misuses** in `grammars/rtl_frontend.ebnf` plus one inline-alternation-`$N` corruption (`RTL-FE-0002`, `event_control_list`). The 15 Category-A list rules no longer expose the raw `{first, rest}` (resp. `{…, first, rest}`) iteration envelope: each is corrected to the canonical extraction-spread `[$N, $M::K*]` (a clean flat list — bare-list rules now emit a top-level array; `kind`/`direction`/`data_type`/etc.-bearing rules keep their leading fields and replace `{first, rest}` with a clean named list field — `ports`/`instances`/`items`/`names`). The rules: `parameter_declaration_sequence`, `port_list`, `port_group`, `genvar_declaration`, `module_instantiation`, `parameter_override_list` (named + positional), `port_connection_list` (named + positional), `net_declaration`, `assignment_target` (concat), `repetition_expr`, `concatenation_expr`, `scoped_identifier`, `enum_type`, `struct_union_field`. Separately, `RTL-FE-0002`: `event_control_list := at lparen event_control_item ( ( comma \| kw_or ) event_control_item )* rparen -> {first: $3, rest: $4}` bound the bare positional `$4` to an **inline alternation** `( comma \| kw_or )` iteration lead — the same emit-time corruption class as `RTL-FE-0001`/`SVPP-0001`/`VHDL-0001` (but **not** a binop level) — so `procedural_block`'s `always_ff`/`always` `event_control` surfaced `"<invalid_sequence_access>"`; the inline alternation is lifted into a new **un-annotated** named rule `event_separator := comma \| kw_or` and `event_control_list` rewritten to `at lparen event_control_item ( event_separator event_control_item )* rparen -> [$3, $4::2*]` (clean `event_control_item` list, no `<invalid_sequence_access>`). Annotation inventory **unchanged at 156 annotations / 74 distinct rules** (the bare-list Category-A rules flip `return_object` → `return_array`; the `{…, items/ports/names: […]}` ones stay `return_object` with a new `normalized_text`; `event_separator` is **un-annotated** so it is not in the inventory — no count delta). Same accept set (no grammar acceptance change — purely the annotation form + the separator lift). The 15 Category-A corrections are a clean shape improvement and are **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`; only `RTL-FE-0002` (the `<invalid_sequence_access>` corruption) is a bug-ledger row. Gate-locked. |
| 2 | 1.0.2 | **RTL-FE-0001 correctness fix (breaking).** The ten `binop_chain` levels no longer emit `"<invalid_sequence_access>"` (and a malformed nested `{level, lhs:["","<op>"], rest:<invalid>}` object) in `rest` for multi-operand input. The five inline operator alternations that were the iteration lead of `( ( a \| b ) next )*` (and corrupted the positional model so the bare `rest: $2` mis-recursed) are lifted into **named** rules `equality_op := eqeq \| ne`, `relational_op := less_equal \| lt \| ge \| gt`, `shift_op := shl \| shr`, `additive_op := plus \| minus`, `multiplicative_op := star \| slash \| percent`, mirroring the gate-locked `rtl_const_expr` RTL-CE-Slice-2 fix and the `systemverilog.ebnf` `binary_operator` op-chain idiom. The five level rules' `binop_chain` annotations are **unchanged** (`{type: "binop_chain", level, lhs: $1, rest: $2}`); only the inline `( a \| b )` became a named op-rule, so `rest` is now the clean `[ [op-envelope, operand] … ]` array (operator token text at `entry[0][1]`, `[]` when no operator). The five `*_op` rules are **un-annotated** alternations, so the annotation inventory is **unchanged at 156 annotations / 74 distinct rules** — only the `binop_chain.rest` shape changed. Same accept set (no grammar acceptance change — purely the alternation lift). Gate-locked. |
| 1.0.0 | 1.0.1 | **RTL-FE-Slice-1..7** — initial 156-annotation baseline. Dispatch wrappers (design_item/package_item/module_item/generate_item), keyword leaves, expression dispatch + 10-rule binop_chain hierarchy, declarations + module structure, parameter/port rules, module instantiation/ports/statements/signals/datatypes mass batch. **NOTE:** the ten `binop_chain` levels' `rest` shape in this baseline was defective for multi-operand input (`RTL-FE-0001`, the inline-alternation-`$N` `"<invalid_sequence_access>"` malformation) — see schema `2` for the correction. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/rtl_frontend.ebnf`) un-annotated except for `rtl_frontend_file -> {type, items}` root. AST dump is the recursive-envelope shape across all rules. |

## Resolved Defects — `RTL-FE-0001` (fixed in release 1.0.2, schema 2)

- **`RTL-FE-0001` — `binop_chain.rest` `<invalid_sequence_access>`
  (`Released`, fixed in parser release `1.0.2` / schema `2`).**
  *Historical (release `1.0.1`, schema `1`):* for any multi-operand
  expression input (e.g. `assign y = a + b;`), the affected
  `binop_chain` level's `rest` surfaced the literal sentinel string
  `"<invalid_sequence_access>"` plus a malformed nested
  `{type: "binop_chain", level: "<level>", lhs: ["", "<op>"], rest:
  <invalid>}` object instead of the clean `(operator, operand)`
  iteration array. Root cause: each level rule
  `<level>_expr := <next> ( ( a | b ) <next> )* -> {…, rest: $2}` had
  the bare positional `rest: $2` reference an **inline alternation
  group** `( a | b )` as the lead element of the `( … <next> )*`
  iteration, the same emit-time defect class fixed for `rtl_const_expr`
  in RTL-CE-Slice-2 (`RTL-CE-0001`) and for `systemverilog_preprocessor`
  (`SVPP-0001`), and still tracked for `vhdl` `binop_chain`. The
  single-operand surface (empty `rest`) was unaffected. **Fix (proven
  RTL-CE-Slice-2 / `systemverilog.ebnf` op-chain idiom):** the five
  inline operator alternations are lifted into **named** rules:

  ```ebnf
  equality_op       := eqeq | ne
  relational_op     := less_equal | lt | ge | gt
  shift_op          := shl | shr
  additive_op       := plus | minus
  multiplicative_op := star | slash | percent
  ```

  and the five level rules now reference the named op-rule (e.g.
  `additive_expr := multiplicative_expr ( additive_op multiplicative_expr )* -> {type: "binop_chain", level: "additive", lhs: $1, rest: $2}`).
  The five `binop_chain` level annotations are **unchanged** — only the
  inline `( a | b )` became a named rule. The parser is regenerated and
  the corrected shape is machine-locked. For input
  `` module m;\nassign y = a + b;\nendmodule\n`` the `additive`-level
  `binop_chain.rest` is now the clean array
  `[ [ [[],"+"], {type:"binop_chain", level:"multiplicative",
  lhs:<b>, rest:[]} ] ]` — one `[op-envelope, operand]` entry per
  operator, the operator token text at `entry[0][1]` (`"+"`), `[]`
  when there is no operator — with **no** `<invalid_sequence_access>`
  anywhere. This is the identical consumer left-fold contract as
  `rtl_const_expr`'s `binop_chain`. The five `*_op` rules are
  **un-annotated** alternations, so the annotation inventory is
  **unchanged at 156 annotations / 74 distinct rules** — only the
  `binop_chain.rest` shape changed and the schema/release version
  bumped. The honest pre-fix history is kept in the
  [Schema Versioning](#schema-versioning) table (schema `1.0.0` row),
  the
  [ten-level `binop_chain` contract](#expressions--the-ten-level-binop_chain-left-fold-contract)
  section, and the binary-addition worked example's schema-`2`
  transition note; tracked (status `Released`) in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (`RTL-FE-0001`).
  Documented in
  [the binary-addition worked example](../rtl_frontend_parser_book/src/examples-binary-addition.md).

## AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT) — 15 Category-A raw-envelope list rules → clean lists; `RTL-FE-0002` `event_control_list` inline-alternation fix; schema 2 → 3

Landed 2026-05-17. The POST-SV-AUDIT static classification pass
(`docs/POST_SV_AUDIT_LEDGER.md`, leaf POST-SV-AUDIT.2.2, tracked
`PGEN-POST-SV-AUDIT-0003`) found **15 static-conclusive Category-A
raw-envelope misuses** plus one inline-alternation-`$N` corruption
(`RTL-FE-0002`) in `grammars/rtl_frontend.ebnf`. They are corrected, the
parser is regenerated, and the manifest inventory is re-locked.

The 15 Category-A corrections are a **clean AST-shape improvement** (no
`<invalid_sequence_access>`, no crash) and are **not** logged in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (that ledger is
reserved for the `<invalid_sequence_access>` corruption/crash class).
`RTL-FE-0002` **is** a released-parser bug (it emitted
`<invalid_sequence_access>` in `procedural_block.event_control`) and is
logged there as its own row.

### The 15 Category-A list rules (raw `{first, rest}` → extraction-spread)

Each rule was a static-conclusive Category-A
pure-single-token-separator (or zero-payload inline-separator) list
rendered as the raw recursive-envelope `{first, rest}` (resp.
`{…, first, rest}`) — `rest` surfaced the raw `[[sep, item], …]`
separator envelope, forcing every consumer to index past the separator
on each iteration. The fix is the canonical extraction-spread idiom
(drop the semantically-irrelevant separator; emit a clean flat list).
Bare-list rules now emit a **top-level array** (`return_object` →
`return_array`); rules with leading discriminator/type fields keep those
fields and replace the `{first, rest}` pair with a single clean named
list field. `≤ 1.0.2` (schema `≤ 2`) shapes are kept as labeled history.

| Rule (`grammars/rtl_frontend.ebnf`) | `≤ 1.0.2` / schema `≤ 2` shape (history) | `1.0.3` / schema `3` shape | Annotation form |
|---|---|---|---|
| `parameter_declaration_sequence` | `{first: $1, rest: $2}` | `[$1, $2::2*]` | `return_object` → `return_array` (top-level `parameter_declaration_group` array) |
| `port_list` | `{first: $1, rest: $2}` | `[$1, $2::2*]` | `return_object` → `return_array` (top-level `port_group` array) |
| `port_group` | `{direction: $1, data_type: $2, packed_range: $3, first: $4, rest: $5}` | `{direction: $1, data_type: $2, packed_range: $3, ports: [$4, $5::3*]}` (the `!port_direction_token` negative-lookahead occupies a positional slot — probe-confirmed) | stays `return_object`, new `normalized_text` |
| `genvar_declaration` | `{first: $2, rest: $3}` | `[$2, $3::2*]` | `return_object` → `return_array` (top-level identifier array) |
| `module_instantiation` | `{module_name: $1, parameters: $2, first: $3, rest: $4}` | `{module_name: $1, parameters: $2, instances: [$3, $4::2*]}` | stays `return_object`, new `normalized_text` |
| `parameter_override_list` (named) | `{kind: "named", first: $2, rest: $3}` | `{kind: "named", items: [$2, $3::3*]}` (the `&dot` zero-width lookahead occupies a positional slot — probe-confirmed) | stays `return_object`, new `normalized_text` |
| `parameter_override_list` (positional) | `{kind: "positional", first: $2, rest: $3}` | `{kind: "positional", items: [$2, $3::3*]}` (the `!dot` lookahead occupies a positional slot — probe-confirmed) | stays `return_object`, new `normalized_text` |
| `port_connection_list` (named) | `{kind: "named", first: $2, rest: $3}` | `{kind: "named", items: [$2, $3::3*]}` (the `&dot` lookahead occupies a positional slot — probe-confirmed) | stays `return_object`, new `normalized_text` |
| `port_connection_list` (positional) | `{kind: "positional", first: $2, rest: $3}` | `{kind: "positional", items: [$2, $3::3*]}` (the `!dot` lookahead occupies a positional slot — probe-confirmed) | stays `return_object`, new `normalized_text` |
| `net_declaration` | `{data_type: $1, packed_range: $2, first: $3, rest: $4}` | `{data_type: $1, packed_range: $2, items: [$3, $4::2*]}` | stays `return_object`, new `normalized_text` |
| `assignment_target` (concat) | `{kind: "concat", first: $2, rest: $3}` | `{kind: "concat", items: [$2, $3::2*]}` | stays `return_object`, new `normalized_text` |
| `repetition_expr` | `{count: $2, first: $4, rest: $5}` | `{count: $2, items: [$4, $5::2*]}` | stays `return_object`, new `normalized_text` |
| `concatenation_expr` | `{first: $2, rest: $3}` | `[$2, $3::2*]` | `return_object` → `return_array` (top-level operand array) |
| `scoped_identifier` | `{first: $1, rest: $2}` | `[$1, $2::2*]` | `return_object` → `return_array` (top-level identifier array) |
| `enum_type` | `{base: $2, packed_range: $3, first: $5, rest: $6}` | `{base: $2, packed_range: $3, items: [$5, $6::2*]}` | stays `return_object`, new `normalized_text` |
| `struct_union_field` | `{data_type: $1, packed_range: $2, first: $3, rest: $4}` | `{data_type: $1, packed_range: $2, names: [$3, $4::2*]}` | stays `return_object`, new `normalized_text` |

In every case the `1.0.3` list is a **clean flat array** of the element
type in source order — there is **no** raw `[[sep, item], …]` envelope,
no `.first` / `.rest` split, and no separator to skip. A consumer
written against `≤ 1.0.2` that walked `.first` + `.rest[][1]` (or, for
the bare-list rules, treated the value as a `{first, rest}` object) must
repin to schema `3` and treat the field (or the rule's whole value, for
the five bare-list rules) as a flat element array. The two
`parameter_override_list` and two `port_connection_list` branches keep
their `kind` discriminator; `port_group`, `module_instantiation`,
`net_declaration`, `assignment_target`, `repetition_expr`, `enum_type`,
and `struct_union_field` keep their leading typed fields and only the
list pair changed.

### `RTL-FE-0002` — `event_control_list` inline-alternation-`$N` corruption (bug-ledger row)

`event_control_list` is the sensitivity-list rule reached as
`procedural_block.event_control` for the `always_ff` / `always` forms.
The `≤ 1.0.2` grammar was

```ebnf
event_control_list := at lparen event_control_item
                       ( ( comma | kw_or ) event_control_item )* rparen
                    -> {first: $3, rest: $4}
```

The bare positional `$4` referenced an **inline alternation**
`( comma | kw_or )` as the lead element of the `( … )*` iteration. This
is the same emit-time root cause as `RTL-FE-0001` / `SVPP-0001` /
`VHDL-0001` (it corrupts the positional model in
`rust/src/ast_pipeline/ast_return_transform.rs` so the rule's own
annotation is mis-recursed) — but it is **not** a `binop_chain` level
(so it was not covered by the closed RTL-FE-0001 binop fix). For
multi-entry sensitivity input (e.g.
`always_ff @(posedge clk or negedge rst) ;`) `event_control` surfaced
`"<invalid_sequence_access>"` instead of the clean event-item list.
**Fix:** the inline alternation is lifted into a new **un-annotated**
named rule and the separator (semantically irrelevant — both `,` and
`or` merely join sensitivity entries) is dropped:

```ebnf
event_separator    := comma | kw_or
event_control_list := at lparen event_control_item
                       ( event_separator event_control_item )* rparen
                    -> [$3, $4::2*]
```

`procedural_block.event_control` is now a clean
`[{edge, expr}, …]` array of `event_control_item` objects (probe-verified
for `always_ff @(posedge clk or negedge rst) ;` — no
`<invalid_sequence_access>` anywhere). `event_separator` is **un-annotated**,
so it does not enter the inventory. `RTL-FE-0002` is tracked (status
`Released`, fixed in parser release `1.0.3` / schema `3`) in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

### Counts and locking

Annotation count: **156** (UNCHANGED — the bare-list Category-A rules
flip `return_object` → `return_array`; the `{…, items/ports/names: […]}`
ones stay `return_object` with a new `normalized_text`; the new
`event_separator` is an **un-annotated** alternation and is not in the
inventory — there is **no count delta**). **74** distinct rules
(UNCHANGED). Same accept set (no grammar acceptance change — purely the
annotation form + the `event_separator` lift). Schema bumped `2 → 3`
because the 15 Category-A rule shapes and
`procedural_block.event_control` changed in a consumer-visible way.
Gate-locked:
`cargo test --lib --features generated_parsers rtl_frontend_ast_shape_contract`
and
`make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate`.

> **Audit-campaign note:** POST-SV-AUDIT.2.2 follows POST-SV-AUDIT.2.1
> (`systemverilog_preprocessor` `macro_formals`, `PGEN-POST-SV-AUDIT-0002`,
> sv_preprocessor 1.0.3 / schema 3). The remaining Category-A
> raw-envelope list rules in `vhdl.ebnf` and `systemverilog.ebnf` are
> tracked separately as their own POST-SV-AUDIT.2.x slices; this release
> corrects `rtl_frontend` only. The 15 Category-A corrections here are a
> clean shape improvement (**not** bug-ledger entries); only the
> `RTL-FE-0002` inline-alternation-`$N` corruption is a released-parser
> bug logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Release 1.0.2 / Contract 1.0.2 Highlights — RTL-FE-0001 correctness fix (binop_chain.rest); schema 1 → 2

Landed 2026-05-16. A worked-example pass surfaced that the `1.0.1`
baseline shipped one shape defect (`RTL-FE-0001`) — the same systemic
inline-operator-alternation-`$N` class fixed for `rtl_const_expr`
(RTL-CE-Slice-2 / `RTL-CE-0001`) and `systemverilog_preprocessor`
(`SVPP-0001`) — that the (root-keys-only) shape-contract regression lock
did not catch. It is fixed, the parser is regenerated, and the manifest
inventory carries a new `assignment_expr` regression sample so the
corrected `binop_chain.rest` shape is now machine-locked.

- **`binop_chain.rest` `<invalid_sequence_access>` (RTL-FE-0001).**
  For any multi-operand expression input, the affected `binop_chain`
  level's `rest` emitted the literal sentinel string
  `"<invalid_sequence_access>"` plus a malformed nested
  `{type: "binop_chain", level: "<level>", lhs: ["", "<op>"], rest:
  <invalid>}` object instead of the clean iteration array. Root cause:
  the ten level rules each had a bare positional `rest: $2` referencing
  an **inline alternation group** `( a | b )` as the lead element of
  the `( ( a | b ) <next> )*` iteration, which corrupts the positional
  model so the rule's own annotation is mis-recursed. **Fix (proven
  RTL-CE-Slice-2 / `systemverilog.ebnf` op-chain idiom):** the five
  inline operator alternations are lifted into **named** rules:

  ```ebnf
  equality_op       := eqeq | ne
  relational_op     := less_equal | lt | ge | gt
  shift_op          := shl | shr
  additive_op       := plus | minus
  multiplicative_op := star | slash | percent

  equality_expr       := relational_expr  ( equality_op       relational_expr     )*
                      -> {type: "binop_chain", level: "equality",       lhs: $1, rest: $2}
  relational_expr     := shift_expr        ( relational_op     shift_expr          )*
                      -> {type: "binop_chain", level: "relational",     lhs: $1, rest: $2}
  shift_expr          := additive_expr     ( shift_op          additive_expr       )*
                      -> {type: "binop_chain", level: "shift",          lhs: $1, rest: $2}
  additive_expr       := multiplicative_expr ( additive_op     multiplicative_expr )*
                      -> {type: "binop_chain", level: "additive",       lhs: $1, rest: $2}
  multiplicative_expr := unary_expr        ( multiplicative_op unary_expr          )*
                      -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
  ```

  The five `binop_chain` level annotations are **unchanged** — only the
  inline `( a | b )` became a named op-rule. The fixed shape for input
  `` module m;\nassign y = a + b;\nendmodule\n`` is, at the `additive`
  level, `{type: "binop_chain", level: "additive", lhs: <a>, rest:
  [ [ [[],"+"], {type:"binop_chain", level:"multiplicative", lhs:<b>,
  rest:[]} ] ]}` — `rest` is the clean array of `[op-envelope,
  operand]` iteration entries, the operator token text at
  `entry[0][1]` (`"+"`), `[]` for no operator — **no**
  `<invalid_sequence_access>` anywhere. This is the identical
  consumer-fold contract as `rtl_const_expr`'s `binop_chain`.

Annotation count: **156** (UNCHANGED — the five `*_op` rules are
**un-annotated** alternations and are not in the inventory). **74**
distinct rules (UNCHANGED). All 156 remain `return_object` /
`return_scalar` as before. Same accept set (no grammar acceptance
change — purely the alternation lift). Schema bumped `1 → 2` because
`binop_chain.rest` changed shape in a consumer-visible way (was the
malformed `<invalid_sequence_access>` + nested object, now the clean
`[op-envelope, operand]` iteration array). Gate-locked:
`cargo test --lib --features generated_parsers rtl_frontend_ast_shape_contract`
and
`make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate`.

> **Systemic note:** the inline-operator-alternation antipattern that
> caused `RTL-FE-0001` is the same root cause fixed for `rtl_const_expr`
> in RTL-CE-Slice-2 (`RTL-CE-0001`,
> `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md`
> Release 1.0.2 systemic note) and for `systemverilog_preprocessor`
> (`SVPP-0001`,
> `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`).
> The same antipattern still exists in `grammars/vhdl.ebnf`
> `binop_chain` levels; that family's correction is tracked separately
> as its own slice + bug-ledger entry. This release fixes rtl_frontend
> only.

## Release 1.0.1 / Contract 1.0.1 Highlights — RTL-FE-Slice-1..7: full grammar typed (164 rules / 156 annotations)

Seven slices landed on 2026-05-14 covering the entire rtl_frontend.ebnf surface:

```ebnf
# File root
rtl_frontend_file        -> {type: "rtl_frontend_file", items}

# Dispatch wrappers (slice 1: 4 rules / 27 annotations)
design_item              -> {kind: "typedef"|"package"|"module"|"semi", body?}    -- 4 kinds
package_item             -> {kind, body?}                                          -- 3 kinds
module_item              -> {kind, body?}                                          -- 10 kinds
generate_item            -> {kind, body?}                                          -- 11 kinds

# Keyword/operator leaves (slice 2: 5 rules / 12 annotations)
parameter_flavor         -> {kind: "parameter"|"localparam"}
port_direction           -> {kind: "input"|"output"|"inout"}
port_direction_token     -> {kind: "input"|"output"|"inout"}
event_edge               -> {kind: "posedge"|"negedge"}
assignment_operator      -> {kind: "blocking"|"nonblocking"}

# Expression dispatch + procedural (slice 3: 6 rules / 21 annotations)
parameter_override       -> {kind: "named"|"positional", name?, value}
procedural_block         -> {kind: "always_comb"|"always_latch"|"always_ff"|"always", ...}
always_star_event        -> {kind: "at_paren_star"|"bare_star"}
conditional_expr         -> {type: "ternary", condition, then_expr, else_expr}    | passthrough
unary_expr               -> {type: "unary", op, expr}                              | passthrough
primary_expr             -> {kind, body?, expr?}                                   -- 6 kinds

# 10-rule binop_chain hierarchy (slice 4: 10 rules / 10 annotations)
logical_or_expr / logical_and_expr / bit_or_expr / bit_xor_expr / bit_and_expr /
equality_expr / relational_expr / shift_expr / additive_expr / multiplicative_expr
                         -> {type: "binop_chain", level, lhs, rest}

# Declarations + module structure (slice 5: 13 rules / 15 annotations)
package_declaration      -> {name, items}
module_declaration       -> {name, imports_pre, parameters, imports_post, ports, items}
generate_region          -> {items}
generate_if              -> {cond, then_body, else_body}
generate_for             -> {genvar, init_var, init_value, condition, step_var, step_value, body}
generate_body            -> {kind: "block"|"single", label?, items|body}
parameter_declaration_statement -> {body}
parameter_declaration_sequence  -> {first, rest}
import_declaration       -> {package, member}
typedef_declaration      -> {data_type, packed_range, name}
genvar_declaration       -> {first, rest}
net_declaration          -> {data_type, packed_range, first, rest}
net_item                 -> {name, dims}
continuous_assign        -> {lvalue, value}

# Parameter + port rules (slice 6: 6 rules / 10 annotations)
parameter_declaration_group  -> {head, tail}
parameter_declaration_head   -> {kind: "typed"|"untyped", flavor, ...}
parameter_declaration_tail   -> {kind, flavor?, data_type?, name, default?}
port_list                    -> {first, rest}
port_group                   -> {direction, data_type, packed_range, first, rest}
port_item                    -> {name, dims}

# Mass batch — module/port/statement/signal/datatype (slice 7: 24+ rules / 59 annotations)
module_instantiation     -> {module_name, parameters, first, rest}
instance_item            -> {name, dims, connections}
parameter_override_list  -> {kind: "named"|"positional", first, rest}
port_connection_list     -> {kind: "named"|"positional", first, rest}
port_connection          -> {kind: "wildcard"|"named"|"positional", ...}
event_control_list       -> {first, rest}
event_control_item       -> {edge, expr}
statement (4 kinds)      -> {kind: "semi"|"block"|"if"|"assignment", ...}
always_ff_statement (4)  -> {kind: "semi"|"block"|"if"|"assignment", ...}
assignment_target (3)    -> {kind: "concat"|"ranged"|"signal", ...}
repetition_expr          -> {count, first, rest}
concatenation_expr       -> {first, rest}
ranged_signal_reference  -> {name, path, msb, lsb}
signal_reference         -> {name, path}
scoped_identifier        -> {first, rest}
signal_path_op (2 kinds) -> {kind: "member"|"index", ...}
data_type (6 kinds)      -> {kind: "enum"|"union"|"struct"|"builtin"|"package"|"named", body}
package_qualified_type   -> {package, name}
builtin_data_type        -> {kind: "bit"|"byte"|"shortint"|"int"|"integer"|"longint"|"logic"|"reg"|"wire"}
enum_type                -> {base, packed_range, first, rest}
enum_base_type           -> {kind: "builtin"|"package"|"named", body}
enum_item                -> {name, value}
union_type / struct_type -> {packed, fields}
struct_union_field       -> {data_type, packed_range, first, rest}
struct_union_field_name  -> {kind: "identifier"|"byte", body?}
packed_range             -> {msb, lsb}
literal (3 kinds)        -> {kind: "based"|"decimal"|"real", text?, body?}
```

The `binop_chain` shape is the consumer-facing left-fold contract; consumers fold `lhs` + `rest` left-associatively to evaluate or print expressions.

Annotation count: **156** (was 1 / pre-typing baseline). Same accept set.

> Header reconciliation: the Highlights header above reads "164 rules /
> 156 annotations". `164` is the **total number of grammar rules** in
> `grammars/rtl_frontend.ebnf`; the typed surface is **156 annotations on
> 74 of those rules**. The inventory-accurate figure used throughout this
> contract (and the per-family book) is **156 annotations / 74 distinct
> rules**; this is a reconciled wording difference, not a contradiction —
> the `156` is identical in both phrasings.

## AST Envelope and Dispatch

This section is the consumer-facing dispatch contract: how a downstream
integrator goes from the host AST-dump call to a typed rtl_frontend tree,
and how to branch on the top-level discriminators. Every shape below is
transcribed from the live inventory
`generated/rtl_frontend_return_annotations.json`
(`version: 1`, `grammar: "rtl_frontend"`, `annotation_count: 156`,
**74 distinct rules**), cross-checked against the embedded copy in
`rust/test_data/ast_shape_contract/rtl_frontend_v1.json` (content-identical
on the `(rule, branch_index, annotation_type, normalized_text)` tuples; the
embedded copy omits only the diagnostic `raw_text` field), and is
consistent with the curated per-rule reference at
`docs/rtl_frontend_parser_book/src/rules-top-level.md`.

### The `AstDumpPayload` envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`, used with grammar family
`rtl_frontend` / profile `default`) return — on success — an
`AstDumpPayload` (defined in `rust/src/embedding_api.rs`, contract in
`rust/docs/EMBEDDING_API_CONTRACT.md`). It is a canonical-JSON payload
string plus truncation metadata, with exactly four fields:

| Field | Type | Meaning |
|---|---|---|
| `dump_json` | string | The canonical (key-sorted) JSON encoding of the typed rtl_frontend AST. Parse this string to obtain the `rtl_frontend_file` root object described below. |
| `truncated` | bool | `false` for a complete dump; `true` when `max_ast_bytes` was exceeded and `dump_json` instead carries the truncation diagnostic envelope. |
| `full_bytes` | int | Byte length of the full encoded AST payload (before any truncation). |
| `emitted_bytes` | int | Byte length actually placed in `dump_json`. Equals `full_bytes` when not truncated. |

When `truncated` is `true`, `dump_json` is replaced by a deterministic
truncation diagnostic envelope (not the AST). That envelope carries
`pgen_dump_contract_version` (currently `1`), `kind:
"pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
"parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`. Consumers
must check `truncated` (or, equivalently, the presence of
`pgen_dump_contract_version` / `kind == "pgen_ast_dump_truncation"` in the
parsed `dump_json`) before treating `dump_json` as an rtl_frontend AST. If
`max_ast_bytes` is too small to fit even the diagnostic envelope, the API
returns `E_INVALID_LIMITS` instead.

> Accuracy note: the live `AstDumpPayload` struct exposes precisely
> `dump_json` / `truncated` / `full_bytes` / `emitted_bytes`. The
> `pgen_dump_contract_version` / `schema_version` / `grammar` / `profile` /
> `root` keys are **not** members of `AstDumpPayload` itself —
> `pgen_dump_contract_version` appears only inside the truncation
> diagnostic envelope, the schema axis is the **AST-dump schema version
> `3`** tracked in [Schema Versioning](#schema-versioning), the grammar
> family is the fixed `rtl_frontend` label, and the profile is the fixed
> `default` profile (see [Stable Integration
> Surface](#stable-integration-surface)). The "root" is the parsed
> `rtl_frontend_file` object documented next. This contract documents the
> surface as it exists in `rust/src/embedding_api.rs`, not an idealized
> envelope.

### The `rtl_frontend_file` root

The parsed `dump_json` is, for a successful rtl_frontend parse, a single
typed root object. Per `grammars/rtl_frontend.ebnf` (line 18–19):

```ebnf
rtl_frontend_file := trivia design_item* trivia
                  -> {type: "rtl_frontend_file", items: $2}
```

```json
{
  "type": "rtl_frontend_file",
  "items": [ /* array of design_item shapes, source order */ ]
}
```

Consumers dispatch on `obj["type"] == "rtl_frontend_file"` at the root,
then iterate `obj["items"]` — each element is one typed `design_item`
object in source order. This is the only rule that carries a `type`
discriminator at the dispatch level; every other dispatcher uses `kind`.

### The 4-branch `design_item` dispatch

`design_item` is the primary top-level dispatcher. It is a 4-branch
`kind`-tagged shape (`grammars/rtl_frontend.ebnf` line 22). Consumers
dispatch on `obj["kind"]`; every branch except `"semi"` carries a `body`
holding the underlying typed shape:

```ebnf
design_item := typedef_declaration -> {kind: "typedef",  body: $1}
             | package_declaration -> {kind: "package",  body: $1}
             | module_declaration  -> {kind: "module",   body: $1}
             | semi                -> {kind: "semi"}
```

| `kind` | `body` shape (fields) | Underlying rule (`grammars/rtl_frontend.ebnf`) |
|---|---|---|
| `"typedef"` | `{data_type, packed_range, name}` — `packed_range` is `[]` when no `[msb:lsb]` is present | `typedef_declaration` (line 117) |
| `"package"` | `{name, items}` — `items` is the `package_item*` array | `package_declaration` (line 27) |
| `"module"` | `{name, imports_pre, parameters, imports_post, ports, items}` — `parameters` / `ports` are `[]` when the optional header clause is absent; `items` is the `module_item*` array | `module_declaration` (line 35) |
| `"semi"` | _(no `body` — a lone top-level `;` separator)_ | `semi` (line 372) |

The `"semi"` branch is the only bodyless one: it carries solely
`{kind: "semi"}` for a stray top-level `;`. Every other branch carries
`{kind, body}` where `body` is the typed shape of the named rule.

### The 10-branch `module_item` dispatch

`module_item` is the in-module construct dispatcher
(`grammars/rtl_frontend.ebnf` line 39). It is a 10-branch `kind`-tagged
shape; every branch except `"semi"` carries `body: $1`:

```ebnf
module_item := parameter_declaration_statement -> {kind: "parameter",            body: $1}
             | import_declaration               -> {kind: "import",              body: $1}
             | typedef_declaration              -> {kind: "typedef",             body: $1}
             | genvar_declaration               -> {kind: "genvar",              body: $1}
             | module_instantiation             -> {kind: "module_instantiation", body: $1}
             | net_declaration                  -> {kind: "net",                 body: $1}
             | continuous_assign                -> {kind: "continuous_assign",   body: $1}
             | procedural_block                 -> {kind: "procedural_block",    body: $1}
             | generate_region                  -> {kind: "generate_region",     body: $1}
             | semi                             -> {kind: "semi"}
```

| `kind` | `body` shape (fields) | Underlying rule (`grammars/rtl_frontend.ebnf`) |
|---|---|---|
| `"parameter"` | `{body}` — wraps a `parameter_declaration_sequence` | `parameter_declaration_statement` (line 76) |
| `"import"` | `{package, member}` — `member` is the imported symbol or `*` wildcard | `import_declaration` (line 114) |
| `"typedef"` | `{data_type, packed_range, name}` | `typedef_declaration` (line 117) |
| `"genvar"` | `[identifier, …]` — clean flat array of the comma-separated genvar identifiers (`1.0.3` / schema `3` — was the raw `{first, rest}` envelope at ≤ `1.0.2` / schema `2`; see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--15-category-a-raw-envelope-list-rules--clean-lists-rtl-fe-0002-event_control_list-inline-alternation-fix-schema-2--3)) | `genvar_declaration` (line 120) |
| `"module_instantiation"` | `{module_name, parameters, instances}` — `parameters` is `[]` when no `#(…)` override; `instances` is the clean flat `instance_item[]` list (`1.0.3` / schema `3` — was `{module_name, parameters, first, rest}` at ≤ `1.0.2` / schema `2`) | `module_instantiation` (line 123) |
| `"net"` | `{data_type, packed_range, items}` — `items` is the clean flat `net_item[]` declarator list (`1.0.3` / schema `3` — was `{data_type, packed_range, first, rest}` at ≤ `1.0.2` / schema `2`) | `net_declaration` (line 144) |
| `"continuous_assign"` | `{lvalue, value}` — `assign lvalue = value;` | `continuous_assign` (line 150) |
| `"procedural_block"` | 4-kind dispatch: `{kind: "always_comb", statement}` / `{kind: "always_latch", statement}` / `{kind: "always_ff", event_control, statement}` / `{kind: "always", event, statement}` | `procedural_block` (line 154) |
| `"generate_region"` | `{items}` — the `generate_item*` array of a `generate … endgenerate` block | `generate_region` (line 50) |
| `"semi"` | _(no `body` — a lone `;` separator)_ | `semi` (line 372) |

### The 11-branch `generate_item` dispatch

`generate_item` is the construct dispatcher used inside a generate region
(`grammars/rtl_frontend.ebnf` line 54). It admits the same construct
families as `module_item` but, instead of `"generate_region"`, admits
`"generate_if"` / `"generate_for"` directly — 11 branches; every branch
except `"semi"` carries `body: $1`:

```ebnf
generate_item := parameter_declaration_statement -> {kind: "parameter",            body: $1}
               | import_declaration               -> {kind: "import",              body: $1}
               | typedef_declaration              -> {kind: "typedef",             body: $1}
               | genvar_declaration               -> {kind: "genvar",              body: $1}
               | module_instantiation             -> {kind: "module_instantiation", body: $1}
               | net_declaration                  -> {kind: "net",                 body: $1}
               | continuous_assign                -> {kind: "continuous_assign",   body: $1}
               | procedural_block                 -> {kind: "procedural_block",    body: $1}
               | generate_if                      -> {kind: "generate_if",         body: $1}
               | generate_for                     -> {kind: "generate_for",        body: $1}
               | semi                             -> {kind: "semi"}
```

| `kind` | `body` shape (fields) | Underlying rule (`grammars/rtl_frontend.ebnf`) |
|---|---|---|
| `"parameter"` | `{body}` — wraps a `parameter_declaration_sequence` | `parameter_declaration_statement` (line 76) |
| `"import"` | `{package, member}` | `import_declaration` (line 114) |
| `"typedef"` | `{data_type, packed_range, name}` | `typedef_declaration` (line 117) |
| `"genvar"` | `[identifier, …]` (clean flat array — `1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`) | `genvar_declaration` (line 120) |
| `"module_instantiation"` | `{module_name, parameters, instances}` (`1.0.3` / schema `3`; was `{module_name, parameters, first, rest}` at ≤ `1.0.2`) | `module_instantiation` (line 123) |
| `"net"` | `{data_type, packed_range, items}` (`1.0.3` / schema `3`; was `{data_type, packed_range, first, rest}` at ≤ `1.0.2`) | `net_declaration` (line 144) |
| `"continuous_assign"` | `{lvalue, value}` | `continuous_assign` (line 150) |
| `"procedural_block"` | 4-kind dispatch (same as for `module_item`) | `procedural_block` (line 154) |
| `"generate_if"` | `{cond, then_body, else_body}` — `else_body` is `[]` when there is no `else` | `generate_if` (line 66) |
| `"generate_for"` | `{genvar, init_var, init_value, condition, step_var, step_value, body}` | `generate_for` (line 69) |
| `"semi"` | _(no `body` — a lone `;` separator)_ | `semi` (line 372) |

The full per-family shapes (declarations, ports, the ten-level
`binop_chain` expression hierarchy, data types, literals) are enumerated
in `docs/rtl_frontend_parser_book/src/rules-top-level.md`; the
machine-checkable enumeration of every `(rule, branch_index,
annotation_type, normalized_text)` tuple is
`generated/rtl_frontend_return_annotations.json` and its embedded copy
`rust/test_data/ast_shape_contract/rtl_frontend_v1.json`. The above
enumerates the full typed surface of contract `1.0.3`
(**156 annotations across 74 distinct rules**, schema version `3`); this
contract section is curated, the inventory artifact is the authoritative
machine-checkable enumeration, and this integration contract wins over the
per-family book if they ever disagree.

## Declarations, Types, Ports, Statements, and Expressions

This section is the consumer-facing per-family shape contract: for every
rtl_frontend rule family it enumerates the `kind` discriminator(s) and the
exact field list the parser emits. Every `kind` value, field name, branch
count, and `level` string below is transcribed from the live inventory
`generated/rtl_frontend_return_annotations.json`
(`version: 1`, `grammar: "rtl_frontend"`, `annotation_count: 156`,
**74 distinct rules**), cross-checked against the embedded shape-contract
manifest `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`
(content-identical on the `(rule, branch_index, annotation_type,
normalized_text)` tuples; the embedded copy omits only the diagnostic
`raw_text` field), and consistent with the curated per-rule reference at
`docs/rtl_frontend_parser_book/src/rules-top-level.md`. The top-level
`rtl_frontend_file` root and the `design_item` / `module_item` /
`generate_item` dispatchers are documented in
[AST Envelope and Dispatch](#ast-envelope-and-dispatch) and are **not
repeated here** — this section references that dispatch and covers the
underlying typed shapes it dispatches into.

Two emission conventions recur:

- **Dispatch rules** emit `{kind: "<branch>", body: $N}` (or per-branch
  named fields). Consumers dispatch on `obj["kind"]`. A bodyless
  `{kind: "semi"}` (and, for `port_connection`, `{kind: "wildcard"}`; for
  `struct_union_field_name`, `{kind: "byte"}`) marks a branch with no
  `body`.
- **Separated-list rules** emit a **clean flat array** of the element
  type in source order — the canonical extraction-spread `[$N, $M::K*]`
  idiom (the semantically-irrelevant `,` / `or` / `::` separator is
  dropped). Bare-list rules (`parameter_declaration_sequence`,
  `port_list`, `genvar_declaration`, `concatenation_expr`,
  `scoped_identifier`) emit a **top-level array**; rules with a leading
  discriminator/type field (`port_group`, `module_instantiation`,
  `parameter_override_list`, `port_connection_list`, `net_declaration`,
  `assignment_target` concat, `repetition_expr`, `enum_type`,
  `struct_union_field`) keep those fields and carry the element list in a
  single named field (`ports` / `instances` / `items` / `names`). This is
  the `1.0.3` / schema `3` shape — the POST-SV-AUDIT Category-A
  correction. At ≤ `1.0.2` / schema `2` these rules emitted the raw
  `{first, rest}` (resp. `{…, first, rest}`) envelope where `rest` was
  the raw `[[sep, item], …]` recursive iteration; that history is kept in
  [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--15-category-a-raw-envelope-list-rules--clean-lists-rtl-fe-0002-event_control_list-inline-alternation-fix-schema-2--3).
  Consumers must repin to schema `3` and treat the field (or the rule's
  whole value, for the bare-list rules) as a flat element array — no
  `.first` / `.rest` split, no separator to skip.

Field selectors below name JSON keys exactly as emitted; a field bound to
an optional grammar element is `[]` when that element is absent (e.g.
`typedef_declaration.packed_range`, `module_declaration.parameters`,
`generate_if.else_body`, `enum_item.value`, `signal_reference.path`).

### Modules, packages, and generate constructs

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `module_declaration` (line 35) | `{name, imports_pre, parameters, imports_post, ports, items}` — `imports_pre` / `imports_post` are the pre- and post-parameter `import_declaration*` arrays; `parameters` is the optional `# ( parameter_declaration_sequence? )` envelope (`[]` when absent); `ports` is the optional ANSI port-list envelope; `items` is the `module_item*` array. |
| `package_declaration` (line 27) | `{name, items}` — `items` is the `package_item*` array. |
| `import_declaration` (line 114) | `{package, member}` — `member` is the imported symbol or `*` wildcard. |
| `typedef_declaration` (line 117) | `{data_type, packed_range, name}` — `packed_range` is `[]` when no `[msb:lsb]` is present. |
| `generate_region` (line 50) | `{items}` — the `generate_item*` array of a `generate … endgenerate` block. |
| `generate_if` (line 66) | `{cond, then_body, else_body}` — `else_body` is `[]` when there is no `else`. |
| `generate_for` (line 69) | `{genvar, init_var, init_value, condition, step_var, step_value, body}`. |
| `generate_body` (line 73, 2 kinds) | `{kind: "block", label, items}` for the `begin … end` form (`label` is `[]` when there is no `: label`) / `{kind: "single", body}` for a single nested generate item. |

### Parameters

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `parameter_declaration_statement` (line 76) | `{body}` — wraps a `parameter_declaration_sequence`. |
| `parameter_declaration_sequence` (line 79) | `[parameter_declaration_group, …]` — clean flat array of the comma-separated `parameter_declaration_group` list (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `parameter_declaration_group` (line 82) | `{head, tail}` — `head` carries the leading flavor/type, `tail` the continued declarators. |
| `parameter_declaration_head` (line 86, 2 kinds) | `{kind: "typed", flavor, data_type, name, default}` / `{kind: "untyped", flavor, name, default}`. |
| `parameter_declaration_tail` (line 90, 4 kinds) | `{kind: "typed_with_flavor", flavor, data_type, name, default}` / `{kind: "untyped_with_flavor", flavor, name, default}` / `{kind: "typed", data_type, name, default}` / `{kind: "untyped", name, default}`. |
| `parameter_flavor` (line 95, 2 kinds) | `{kind: "parameter"}` / `{kind: "localparam"}` — bare `{kind}` keyword leaf. |
| `parameter_override` (line 133, 2 kinds) | `{kind: "named", name, value}` / `{kind: "positional", value}`. |
| `parameter_override_list` (line 129, 2 kinds) | `{kind: "named", items}` / `{kind: "positional", items}` — `items` is the clean flat `parameter_override[]` list (`1.0.3` / schema `3`; was `{kind, first, rest}` at ≤ `1.0.2`). |

The `default` field on every parameter shape is `[]` when no
`= <expr>` initializer is present.

### Ports

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `port_list` (line 98) | `[port_group, …]` — clean flat array of the comma-separated `port_group` list (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `port_group` (line 101) | `{direction, data_type, packed_range, ports}` — `direction` is the typed `port_direction`; `data_type` / `packed_range` are `[]` when omitted; `ports` is the clean flat `port_item[]` declarator list (`1.0.3` / schema `3`; was `{direction, data_type, packed_range, first, rest}` at ≤ `1.0.2`). |
| `port_item` (line 104) | `{name, dims}` — `dims` is `[]` for an unpacked-scalar port. |
| `port_direction` (line 107, 3 kinds) | `{kind: "input"}` / `{kind: "output"}` / `{kind: "inout"}` — bare `{kind}` keyword leaf. |
| `port_direction_token` (line 110, 3 kinds) | `{kind: "input"}` / `{kind: "output"}` / `{kind: "inout"}` — the negative-lookahead guard token used in the port-continuation iteration; same `kind` set as `port_direction`. |

### Net / signal / instance declarations and port connections

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `net_declaration` (line 144) | `{data_type, packed_range, items}` — `items` is the clean flat `net_item[]` declarator list (`1.0.3` / schema `3`; was `{data_type, packed_range, first, rest}` at ≤ `1.0.2`). |
| `net_item` (line 147) | `{name, dims}` — `dims` is `[]` for a scalar net. |
| `genvar_declaration` (line 120) | `[identifier, …]` — clean flat array of the comma-separated genvar identifiers (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `continuous_assign` (line 150) | `{lvalue, value}` — `assign lvalue = value;`. |
| `module_instantiation` (line 123) | `{module_name, parameters, instances}` — `parameters` is the optional `#( parameter_override_list? )` envelope (`[]` when absent); `instances` is the clean flat `instance_item[]` list (`1.0.3` / schema `3`; was `{module_name, parameters, first, rest}` at ≤ `1.0.2`). |
| `instance_item` (line 126) | `{name, dims, connections}` — `dims` is `[]` for a non-array instance; `connections` is the `port_connection_list`. |
| `port_connection_list` (line 136, 2 kinds) | `{kind: "named", items}` / `{kind: "positional", items}` — `items` is the clean flat `port_connection[]` list (`1.0.3` / schema `3`; was `{kind, first, rest}` at ≤ `1.0.2`). |
| `port_connection` (line 140, 3 kinds) | `{kind: "wildcard"}` (the `.*` form, **bodyless**) / `{kind: "named", name, value}` / `{kind: "positional", value}`. |

### Procedural / always blocks and statements

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `procedural_block` (line 154, 4 kinds) | `{kind: "always_comb", statement}` / `{kind: "always_latch", statement}` / `{kind: "always_ff", event_control, statement}` / `{kind: "always", event, statement}`. |
| `always_star_event` (line 159, 2 kinds) | `{kind: "at_paren_star"}` (`@(*)`) / `{kind: "bare_star"}` (`@*`). |
| `event_control_list` (line 162) | `[event_control_item, …]` — clean flat array of the comma/`or`-separated `event_control_item` list (`1.0.3` / schema `3`; was the corrupt `{first, rest}` that surfaced `"<invalid_sequence_access>"` at ≤ `1.0.2` — the `RTL-FE-0002` inline-alternation fix, separator lifted to the un-annotated `event_separator` rule; see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--15-category-a-raw-envelope-list-rules--clean-lists-rtl-fe-0002-event_control_list-inline-alternation-fix-schema-2--3)). |
| `event_control_item` (line 165) | `{edge, expr}` — `edge` is the typed `event_edge` (`[]` for a level-sensitive signal). |
| `event_edge` (line 168, 2 kinds) | `{kind: "posedge"}` / `{kind: "negedge"}` — bare `{kind}` keyword leaf. |
| `statement` (line 172, 4 kinds) | `{kind: "semi"}` (bodyless) / `{kind: "block", label, items}` / `{kind: "if", cond, then_body, else_body}` / `{kind: "assignment", lvalue, operator, value}`. |
| `always_ff_statement` (line 178, 4 kinds) | Same 4-kind shape as `statement` (`"semi"` / `"block"` / `"if"` / `"assignment"`); the `always_ff` body grammar restricts the assignment operator to the nonblocking arrow. |
| `assignment_operator` (line 185, 2 kinds) | `{kind: "blocking"}` (`=`) / `{kind: "nonblocking"}` (`<=`) — bare `{kind}` keyword leaf. |
| `assignment_target` (line 189, 3 kinds) | `{kind: "concat", items}` (`{a, b}` LHS — `items` is the clean flat target list; `1.0.3` / schema `3`, was `{kind: "concat", first, rest}` at ≤ `1.0.2`) / `{kind: "ranged", body}` / `{kind: "signal", body}`. |

For `"if"` shapes (`statement` / `always_ff_statement`), `else_body` is
`[]` when there is no `else`. For `"block"`, `label` is `[]` when the
`begin` has no `: label`.

### Expressions — the ten-level `binop_chain` left-fold contract

The rtl_frontend expression grammar is a **ten-level
operator-precedence cascade** (`grammars/rtl_frontend.ebnf` lines
200–218), entered through `rtl_expr` → `conditional_expr` and bottoming
out at `unary_expr` → `primary_expr`. Each of the ten binary levels emits
the **same** typed shape — a `binop_chain` object — so a single consumer
fold routine handles the entire binary-expression tree:

```ebnf
rtl_expr            := conditional_expr
conditional_expr    := logical_or_expr ? conditional_expr : conditional_expr
                    -> {type: "ternary", condition: $1, then_expr: $3, else_expr: $5}
                     | logical_or_expr                                   -- passthrough ($1)

# Named operator rules — un-annotated alternations (the RTL-FE-0001 fix,
# schema 2; not in the 156-annotation inventory):
equality_op         := eqeq | ne
relational_op       := less_equal | lt | ge | gt
shift_op            := shl | shr
additive_op         := plus | minus
multiplicative_op   := star | slash | percent

logical_or_expr     := logical_and_expr ( logical_or  logical_and_expr )*
                    -> {type: "binop_chain", level: "logical_or",     lhs: $1, rest: $2}
logical_and_expr    := bit_or_expr      ( logical_and bit_or_expr     )*
                    -> {type: "binop_chain", level: "logical_and",    lhs: $1, rest: $2}
bit_or_expr         := bit_xor_expr     ( bit_or      bit_xor_expr    )*
                    -> {type: "binop_chain", level: "bit_or",         lhs: $1, rest: $2}
bit_xor_expr        := bit_and_expr     ( bit_xor     bit_and_expr    )*
                    -> {type: "binop_chain", level: "bit_xor",        lhs: $1, rest: $2}
bit_and_expr        := equality_expr    ( bit_and     equality_expr   )*
                    -> {type: "binop_chain", level: "bit_and",        lhs: $1, rest: $2}
equality_expr       := relational_expr  ( equality_op       relational_expr     )*
                    -> {type: "binop_chain", level: "equality",       lhs: $1, rest: $2}
relational_expr     := shift_expr       ( relational_op     shift_expr          )*
                    -> {type: "binop_chain", level: "relational",     lhs: $1, rest: $2}
shift_expr          := additive_expr    ( shift_op          additive_expr       )*
                    -> {type: "binop_chain", level: "shift",          lhs: $1, rest: $2}
additive_expr       := multiplicative_expr ( additive_op    multiplicative_expr )*
                    -> {type: "binop_chain", level: "additive",       lhs: $1, rest: $2}
multiplicative_expr := unary_expr       ( multiplicative_op unary_expr          )*
                    -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
```

> **`RTL-FE-0001` (resolved in `1.0.2` / schema `2`).** At release
> `1.0.1` / schema `1` the five lower levels used **inline operator
> alternations** as the iteration lead — `equality_expr := relational_expr
> ( ( == | != ) relational_expr )*`, and likewise for `relational` /
> `shift` / `additive` / `multiplicative`. A bare positional `rest: $2`
> referencing an inline `( a | b )` group corrupts the positional model
> (`rust/src/ast_pipeline/ast_return_transform.rs`), so for any
> multi-operand input the level's `rest` emitted
> `"<invalid_sequence_access>"` plus a malformed nested
> `{level, lhs:["","<op>"], rest:<invalid>}` object. **Fixed in `1.0.2`**
> by lifting the five inline alternations into the **named, un-annotated**
> op-rules shown above (`equality_op` / `relational_op` / `shift_op` /
> `additive_op` / `multiplicative_op`) — the proven RTL-CE-Slice-2 /
> `systemverilog.ebnf` `binary_operator` idiom. The five `binop_chain`
> level annotations are **unchanged**; only the inline `( a | b )` became
> a named rule, so `rest` is now the clean `[ [op-envelope, operand] … ]`
> array (operator token text at `entry[0][1]`, `[]` for no operator).
> Because the five `*_op` rules are un-annotated, the annotation
> inventory is **unchanged at 156 annotations / 74 distinct rules**. The
> honest pre-fix history is kept here, in the
> [Schema Versioning](#schema-versioning) table, and in the
> binary-addition worked example; tracked (status `Released`) as
> `RTL-FE-0001` in
> `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

The ten levels, in precedence order (loosest binding first), each a
`return_object` annotation on **branch 0** emitting
`{type: "binop_chain", level, lhs: $1, rest: $2}`:

| Level (`level`) | Rule (`grammars/rtl_frontend.ebnf`) | Operators |
|---|---|---|
| `"logical_or"` | `logical_or_expr` (line 200) | `\|\|` |
| `"logical_and"` | `logical_and_expr` (line 202) | `&&` |
| `"bit_or"` | `bit_or_expr` (line 204) | `\|` |
| `"bit_xor"` | `bit_xor_expr` (line 206) | `^` |
| `"bit_and"` | `bit_and_expr` (line 208) | `&` |
| `"equality"` | `equality_expr` (line 210) | `==` / `!=` |
| `"relational"` | `relational_expr` (line 212) | `<=` / `<` / `>=` / `>` |
| `"shift"` | `shift_expr` (line 214) | `<<` / `>>` |
| `"additive"` | `additive_expr` (line 216) | `+` / `-` |
| `"multiplicative"` | `multiplicative_expr` (line 218) | `*` / `/` / `%` |

These are exactly the ten rules that emit `{type: "binop_chain", …}` in
`generated/rtl_frontend_return_annotations.json` (10 rules / 10
annotations, one per rule). No other rtl_frontend rule emits a
`binop_chain`.

**Consumer left-fold rule (normative).** Every level emits
`{type: "binop_chain", level, lhs, rest}` where `lhs` is the leading
operand — itself a `binop_chain` of the next-tighter level (or, at
`multiplicative_expr`, a `unary_expr` shape) — and `rest` is a **clean
array of iteration entries**, one per `(op operand)` repetition of
`<next> ( <NAMED_op> <next> )*`. Each entry is a two-element array:
`entry[0]` is the **operator envelope** of the named op-rule (e.g.
`additive_op := plus | minus`), with the operator **token text at
`entry[0][1]`** (`"+"`, `"-"`, …) and an empty `trivia` at
`entry[0][0]` (`[]` when there is no leading trivia); `entry[1]` is the
right-hand operand — the next-tighter level's `binop_chain`. For input
`assign y = a + b;` the `additive`-level `rest` is the single entry
`[ [ [], "+" ], {type:"binop_chain", level:"multiplicative",
lhs:<b>, rest:[]} ]`. This is the **identical** consumer-fold contract
as `rtl_const_expr`'s `binop_chain` (operator at `entry[0][1]`). It is
the `RTL-FE-0001` corrected, gate-locked shape — at `1.0.1` / schema
`1` this field was the malformed `<invalid_sequence_access>` + nested
object (see
[Resolved Defects — `RTL-FE-0001`](#resolved-defects--rtl-fe-0001-fixed-in-release-102-schema-2)
and the binary-addition worked example). **Consumers MUST fold `rest`
left-associatively onto `lhs`**: evaluate `lhs`, then for each entry in
`rest` (in array order) apply the operator at `entry[0][1]` with the
running result as the left side and `entry[1]` as the right side. This
left-fold is identical at all ten levels by construction, so one fold
routine walks the whole binary-expression tree. All ten levels iterate
`*`, so `rest` may hold **any number** of entries (including zero — a
single operand surfaces as a `binop_chain` whose `rest` is `[]`).

**There is NO `sign` field on any rtl_frontend `binop_chain` level**
(unlike VHDL's `"additive"` level / `simple_expression`, which carries a
leading-sign field). In rtl_frontend the prefix operators are factored
into the separate `unary_expr` tier *below* `multiplicative_expr` — they
never appear inside a `binop_chain`. Consumers must not look for a `sign`
key on any rtl_frontend expression level.

#### Ternary, unary, and primary operands

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `conditional_expr` (line 197, 2 forms) | `{type: "ternary", condition, then_expr, else_expr}` for the `c ? t : e` form; **passthrough** (a `return_scalar` annotation, `-> $1`) when there is no `?`, so a non-ternary expression surfaces directly as its `logical_or_expr` `binop_chain` with **no `conditional_expr` wrapper**. |
| `unary_expr` (line 221, 5 forms) | `{type: "unary", op: "plus", expr}` / `{op: "minus"}` / `{op: "logical_not"}` (`!`) / `{op: "bit_not"}` (`~`); the fifth branch is **passthrough** (`return_scalar`, `-> $1`) — a non-prefixed operand surfaces directly as its `primary_expr` with **no `unary` wrapper**. |
| `primary_expr` (line 228, 6 kinds) | `{kind: "repetition", body}` / `{kind: "concatenation", body}` / `{kind: "ranged_signal", body}` / `{kind: "signal", body}` / `{kind: "literal", body}` / `{kind: "parens", expr}`. |

Because `conditional_expr` and `unary_expr` are **passthrough** when
their distinguishing syntax is absent, any expression slot may hold a
`binop_chain`, a `ternary`, a `unary`, **or directly** a `primary_expr`
`{kind, …}` object. Consumers must dispatch on the presence of `type`
vs. `kind`: a `type` of `"binop_chain"` / `"ternary"` / `"unary"`
selects the corresponding shape, while an object carrying `kind` (no
`type`) is a bare `primary_expr` that flowed through both passthroughs.

#### Signal references and operand leaves

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `signal_reference` (line 244) | `{name, path}` — `name` is the `scoped_identifier`; `path` is the `signal_path_op*` array (`[]` for a plain signal). |
| `ranged_signal_reference` (line 241) | `{name, path, msb, lsb}` — a part-select `sig[msb:lsb]`. |
| `scoped_identifier` (line 247) | `[identifier, …]` — clean flat array of the `pkg::name` scope-resolution chain; a single-element array for an unqualified identifier (`1.0.3` / schema `3`; was `{first, rest}` with `rest` `[]` for an unqualified identifier at ≤ `1.0.2`). |
| `signal_path_op` (line 250, 2 kinds) | `{kind: "member", name}` (`.field`) / `{kind: "index", index}` (`[expr]`). |
| `concatenation_expr` (line 238) | `[operand, …]` — clean flat array of the `{a, b, …}` operands (`1.0.3` / schema `3`; was `{first, rest}` at ≤ `1.0.2`). |
| `repetition_expr` (line 235) | `{count, items}` — `{count{a, …}}`; `items` is the clean flat operand list (`1.0.3` / schema `3`; was `{count, first, rest}` at ≤ `1.0.2`). |
| `literal` (line 301, 3 kinds) | `{kind: "based", text}` / `{kind: "decimal", text}` / `{kind: "real", body}`. |

### Data types

| Rule (`grammars/rtl_frontend.ebnf`) | Shape |
|---|---|
| `data_type` (line 253, 6 kinds) | `{kind: "enum", body}` / `{kind: "union", body}` / `{kind: "struct", body}` / `{kind: "builtin", body}` / `{kind: "package", body}` / `{kind: "named", body}`. |
| `builtin_data_type` (line 265, 9 kinds) | bare `{kind}` for `"bit"`, `"byte"`, `"shortint"`, `"int"`, `"integer"`, `"longint"`, `"logic"`, `"reg"`, `"wire"`. |
| `package_qualified_type` (line 260) | `{package, name}` — `pkg::type_name`. |
| `enum_type` (line 275) | `{base, packed_range, items}` — `base` is the typed `enum_base_type`; `items` is the clean flat `enum_item[]` list (`1.0.3` / schema `3`; was `{base, packed_range, first, rest}` at ≤ `1.0.2`). |
| `enum_base_type` (line 278, 3 kinds) | `{kind: "builtin", body}` / `{kind: "package", body}` / `{kind: "named", body}`. |
| `enum_item` (line 282) | `{name, value}` — `value` is `[]` when there is no `= <expr>`. |
| `struct_type` (line 288) | `{packed, fields}` — `packed` is `[]` when the struct is unpacked; `fields` is the `struct_union_field` list. |
| `union_type` (line 285) | `{packed, fields}` — same shape as `struct_type`. |
| `struct_union_field` (line 291) | `{data_type, packed_range, names}` — `names` is the clean flat field-name declarator list (`1.0.3` / schema `3`; was `{data_type, packed_range, first, rest}` at ≤ `1.0.2`). |
| `struct_union_field_name` (line 294, 2 kinds) | `{kind: "identifier", body}` / `{kind: "byte"}` (the `byte` reserved-word field-name form, **bodyless**). |
| `packed_range` (line 297) | `{msb, lsb}` — `[msb:lsb]`. |

`named_data_type` (a bare identifier type reference,
`grammars/rtl_frontend.ebnf` line 263) is **un-annotated** — it surfaces
through the recursive envelope of its `identifier`, reached as
`data_type.body` when `data_type.kind == "named"`.

The above enumerates the full typed surface of contract `1.0.3`
(**156 annotations across 74 distinct rules**, schema version `3`).
This contract section is curated; the authoritative machine-checkable
enumeration of every `(rule, branch_index, annotation_type,
normalized_text)` tuple is
`generated/rtl_frontend_return_annotations.json` and its embedded copy
`rust/test_data/ast_shape_contract/rtl_frontend_v1.json`. The per-rule
reference at `docs/rtl_frontend_parser_book/src/rules-top-level.md`
mirrors these family groupings; if any disagree, the inventory artifact
wins, and this integration contract wins over the book.

## Companion Documentation — rtl_frontend Parser Integration mdBook

This contract is the **downstream integration surface**: the host-API
envelope, the dispatch/rule-family shapes a consumer compiles against, and
the release/schema axes. It does not duplicate the per-rule walkthroughs or
worked examples — those live in the companion artifacts below. Each surface
is authoritative for a different thing; consult the matching one and respect
the precedence order stated at the end of this section.

| Surface | Path | Authoritative for |
|---|---|---|
| **This contract** | `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` | The downstream integration surface: AST-dump envelope, `rtl_frontend_file` root, the `design_item` / `module_item` / `generate_item` dispatch, and the per-family rule shapes (declarations, parameters, ports, statements, the ten-level `binop_chain` expression hierarchy, data types, literals). See [AST Envelope and Dispatch](#ast-envelope-and-dispatch) and [Declarations, Types, Ports, Statements, and Expressions](#declarations-types-ports-statements-and-expressions). |
| **Per-parser mdBook** | `docs/rtl_frontend_parser_book/` (source `src/*.md`; tracked HTML at `docs/rtl_frontend_parser_book-html/`) | The per-rule reference and teaching surface: build recipe, public API, AST-envelope walkthrough, every rule shape, per-feature worked examples, schema-versioning timeline, glossary, changelog index. Curated, not machine-checked. Listed in `README.md` § "Per-Parser Integration Reference Books". |
| **Shape-contract manifest** | `rust/test_data/ast_shape_contract/rtl_frontend_v1.json` | The machine-checkable shape lock embedded in the regression test. Content-identical to the live inventory on the `(rule, branch_index, annotation_type, normalized_text)` tuples (the embedded copy omits only the diagnostic `raw_text` field). Drift fails the AST-shape-contract test. |
| **Declared-annotation inventory** | `generated/rtl_frontend_return_annotations.json` | The live machine-checkable enumeration of every typed-shape annotation the rtl_frontend grammar emits (`version: 1`, `grammar: "rtl_frontend"`, `annotation_count: 156`, **74 distinct rules**). The generator-side source of truth for the typed surface. |
| **Embedding-API contract** | `rust/docs/EMBEDDING_API_CONTRACT.md` | The canonical host-API truth: the `AstDumpPayload` struct (`dump_json` / `truncated` / `full_bytes` / `emitted_bytes`), the entry-point signatures, the truncation diagnostic envelope, and the stable diagnostics. The struct shape this contract documents is transcribed from there. |
| **Released-parser bug ledger** | `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | The accepted-bug log for the released rtl_frontend parser; `RTL-FE-0001` (status `Released`, fixed in parser release `1.0.2` / schema `2`) and `RTL-FE-0002` (the `event_control_list` inline-alternation-`$N` corruption, status `Released`, fixed in parser release `1.0.3` / schema `3`) live there. The 15 POST-SV-AUDIT Category-A list-shape corrections are a clean shape improvement and are **not** bug-ledger rows (see [AST-Shape Corrections — 1.0.3](#ast-shape-corrections--103-post-sv-audit--15-category-a-raw-envelope-list-rules--clean-lists-rtl-fe-0002-event_control_list-inline-alternation-fix-schema-2--3)). Consult before integrating around a suspected parser defect; file new accepted bugs here per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`. |

Precedence when surfaces disagree (highest first): the **embedding-API
contract** (`rust/docs/EMBEDDING_API_CONTRACT.md`) wins for the host-API /
`AstDumpPayload` truth; the **declared-annotation inventory**
(`generated/rtl_frontend_return_annotations.json`) and its embedded
shape-contract manifest copy win for the exact typed-shape enumeration;
**this integration contract** wins over the **per-parser mdBook** for
downstream compliance. Report any disagreement as a documentation bug
rather than silently coding to the lower-precedence surface.

### Gate Recipe

The exact, copy-pasteable per-family commands a downstream integrator or
releaser runs. Each is verified against the repo (`rust/Makefile`,
`docs/rtl_frontend_parser_book/src/build-recipe.md`,
`rust/src/ast_shape_contract.rs`); none are invented — do not substitute
flags.

**1. On-demand parser regen.** The rtl_frontend parser is on-demand-only
(not in the default `cargo test --features generated_parsers` build).
Build `ast_pipeline`, then regenerate the parser from
`grammars/rtl_frontend.ebnf` (run from `rust/`, per
`docs/rtl_frontend_parser_book/src/build-recipe.md` § "Cold-clone build"):

```bash
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/rtl_frontend.ebnf \
    --generate-parser --output ../generated/rtl_frontend_parser.rs
```

To wire the regenerated parser into a cargo build, point
`PGEN_RTL_FRONTEND_PARSER_PATH` at the absolute path of the generated file
before `cargo build --release --features generated_parsers` (see
`docs/rtl_frontend_parser_book/src/build-recipe.md` § "Wiring into a
downstream Cargo build").

**2. Per-family book gate.** Builds the rtl_frontend parser book and
verifies the tracked HTML landing pages (Makefile target
`rtl_frontend_parser_book_gate`, `rust/Makefile` line 738):

```bash
make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_parser_book_gate
```

**3. AST-shape-contract regression lock.** With the generated backend wired
in (`PGEN_RTL_FRONTEND_PARSER_PATH` exported), run the shape-contract test
that diffs the running generated parser against
`rust/test_data/ast_shape_contract/rtl_frontend_v1.json` (test fn
`rtl_frontend_ast_shape_contract_holds_against_running_generated_parser` in
the `pgen::ast_shape_contract` library module,
`rust/src/ast_shape_contract.rs` line 763):

```bash
cargo test --lib --features generated_parsers rtl_frontend_ast_shape_contract
```

The substring `rtl_frontend_ast_shape_contract` selects exactly the
`rtl_frontend_ast_shape_contract_holds_against_running_generated_parser`
test. Any drift between the running parser's emitted shapes and the locked
manifest fails this test, surfacing the change before release.

**4. Family closure / proof gates.** Anyone publishing a parser-release
version bump also runs the closure/proof gates enumerated in
[Validation / Release Gates](#validation--release-gates) (e.g. the public
host-API stability gate `make -C rust SHELL=/bin/bash embedding_api_gate`
and the per-family contract gate
`make -C rust SHELL=/opt/homebrew/bin/bash rtl_frontend_generated_contract_gate`).
That section is the full list; it is not repeated here.

## Glossary

Contract-scoped definitions of the terms a downstream integrator needs to
read this document. Where a term has a normative definition, this contract
is authoritative; the per-parser book's
[glossary](../rtl_frontend_parser_book/src/glossary.md) paraphrases the
same terms for quick lookup. Numbers below are pinned to contract `1.0.3` /
schema `3` / **156 annotations across 74 distinct rules**.

- **`AstDumpPayload`** — the success return of the rtl_frontend AST-dump
  host entry points (defined in `rust/src/embedding_api.rs`, contract in
  `rust/docs/EMBEDDING_API_CONTRACT.md`). A canonical-JSON payload string
  plus truncation metadata, with **exactly four fields**: `dump_json`,
  `truncated`, `full_bytes`, `emitted_bytes`. It does **not** carry
  `root` / `schema_version` / `grammar` / `profile` members — see
  [The `AstDumpPayload` envelope](#the-astdumppayload-envelope) for the
  precise accuracy note.
- **`dump_json`** — the `AstDumpPayload` field holding the canonical
  (key-sorted) JSON encoding of the typed rtl_frontend AST. Parse this
  string to obtain the `rtl_frontend_file` root object. When `truncated`
  is `true` this string is replaced by the truncation diagnostic envelope,
  not the AST.
- **Truncation diagnostic envelope** — the deterministic JSON object that
  replaces the AST in `dump_json` when `max_ast_bytes` is exceeded. It
  carries `pgen_dump_contract_version` (currently `1`), `kind:
  "pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
  "parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`.
  Consumers must check `truncated` (or detect `kind ==
  "pgen_ast_dump_truncation"`) before treating `dump_json` as an
  rtl_frontend AST.
- **AST-dump schema version** — the integer version axis tracking the AST
  output shape, currently `3`, pinned by this contract (see
  [Schema Versioning](#schema-versioning)). It is **not** a field of
  `AstDumpPayload`; it is the contract-tracked axis. Bumped only when the
  emitted shape changes in a way consumers may need to adapt to (new
  annotation on a previously-unannotated rule, restructured annotation,
  user-visible grammar-shape change). Pure perf work / internal codegen
  reorganization do not bump it.
- **Parser release version** — the parser library's release identity,
  currently `1.0.3`. Bumped on every functional change (bug fixes, perf
  work, grammar changes). Moves independently of the schema version.
- **`design_item` / `module_item` / `generate_item` dispatch** — the
  three top-level `kind`-tagged dispatchers. `design_item` is the primary
  top-level dispatcher (4 branches: `"typedef"`, `"package"`, `"module"`,
  `"semi"`); `module_item` is the in-module construct dispatcher (10
  branches); `generate_item` is the in-generate-region dispatcher (11
  branches — same families as `module_item` but admits `"generate_if"` /
  `"generate_for"` instead of `"generate_region"`). Every branch except
  the bodyless `"semi"` carries a `body`. Every parse roots at
  `{type: "rtl_frontend_file", items: [...]}`; each element of `items` is
  one `design_item` object. See
  [The 4-branch `design_item` dispatch](#the-4-branch-design_item-dispatch),
  [The 10-branch `module_item` dispatch](#the-10-branch-module_item-dispatch),
  and [The 11-branch `generate_item` dispatch](#the-11-branch-generate_item-dispatch).
- **Ten-level `binop_chain` left-fold** — the consumer-facing contract for
  rtl_frontend's ten-level operator-precedence cascade (`logical_or_expr`
  → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr`
  → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr`
  → `multiplicative_expr`). Every level emits
  `{type: "binop_chain", level, lhs, rest}` (10 rules / 10 annotations,
  one per rule); consumers MUST fold `rest` left-associatively onto
  `lhs`. There is **no `sign` field** on any level — prefix operators are
  factored into the separate `unary_expr` tier below `multiplicative_expr`.
  One fold routine walks the whole binary-expression tree. See
  [Expressions — the ten-level `binop_chain` left-fold contract](#expressions--the-ten-level-binop_chain-left-fold-contract).
- **Shape-contract manifest** — the embedded machine-checkable shape lock
  `rust/test_data/ast_shape_contract/rtl_frontend_v1.json`.
  Content-identical to the declared-annotation inventory on the
  `(rule, branch_index, annotation_type, normalized_text)` tuples (omits
  only the diagnostic `raw_text` field). Drift fails the
  `rtl_frontend_ast_shape_contract_holds_against_running_generated_parser`
  regression test (see [Gate Recipe](#gate-recipe)).
- **Declared-annotation inventory** — the live machine-checkable
  enumeration of every typed-shape annotation the rtl_frontend grammar
  emits: `generated/rtl_frontend_return_annotations.json` (`version: 1`,
  `grammar: "rtl_frontend"`, `annotation_count: 156`, **74 distinct
  rules**). The generator-side source of truth for the typed surface;
  mirrored by the embedded shape-contract manifest copy. (The grammar has
  164 total rules; 74 of them carry the 156 typed-shape annotations — see
  the Highlights header-reconciliation note.)
- **Recursive envelope** — the default JSON shape produced by
  un-annotated rules: a recursive composition of arrays (sequences,
  quantified iterations, the `binop_chain` `rest` op-iteration tail),
  strings (terminal/regex leaves), and matched-branch passthroughs (for
  alternations). Un-matched optionals are the empty array `[]`, never
  `null`. It is what a consumer reaches when descending below the typed
  surface (identifier tokens; the `binop_chain` `rest` iteration; the
  un-annotated `named_data_type` reached as `data_type.body` when
  `data_type.kind == "named"`; the few utility rules with no per-rule
  annotation).
- **Generic host AST-dump surface** — the
  `parse_grammar_profile_ast_dump*` family
  (`parse_grammar_profile_ast_dump`,
  `parse_grammar_profile_ast_dump_with_limits`, the `*_result` and
  `*_named` forms). The grammar-agnostic entry points that, for the
  `rtl_frontend` grammar + `default` profile, return the
  `AstDumpPayload`. rtl_frontend has **no** named-convenience entry point
  (unlike VHDL's `parse_vhdl_1076_2019_ast_dump`); the generic family
  used with grammar family `rtl_frontend` / profile `default` is the
  integration surface. Signatures are in
  `rust/docs/EMBEDDING_API_CONTRACT.md`; the stable entry-point list is
  in [Stable Integration Surface](#stable-integration-surface).

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `rtl_frontend` is an `In Progress` family in the live tracker. The current grammar covers the synthesizable RTL subset; the full IEEE 1800 SystemVerilog surface is **out of scope** — see the `systemverilog` family for that.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
