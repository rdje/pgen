# POST-SV-AUDIT classification ledger (POST-SV-AUDIT.1)

Date: 2026-05-17
Task tree: `docs/tasks/POST-SV-AUDIT.md` (leaf `POST-SV-AUDIT.1`)

## Purpose

Static grammar-only classification of every `{… first:$N …}` /
`{… lhs:$N …}` / `{… rest:$N …}` annotation, and any bare positional
`$N` referencing a quantified `( … )*` / `( … )+` iteration, across
the six product grammars. This ledger is the evidence-based worklist
that `POST-SV-AUDIT.2` (fixes) and `POST-SV-AUDIT.3` (Cat-C/residual
close-out) consume. No grammars were edited; no parsers/probes/builds
were run. Suspects requiring empirical confirmation are flagged for
the parent to probe.

## The A / B / C taxonomy (concise)

For a rule whose RHS contains a quantified iteration with a leading
element, whose annotation references the iteration positionally:

- Category A — pure list with a separator: `X ( SEP X )*` where `SEP`
  carries no semantic payload the consumer needs (comma, `or`, `|`,
  `dot`, `semi`, an inline `( a | b )` separator-alternation). Correct
  annotation = an extraction-spread that drops the separator
  (`[$N, $M::K*]`). MISUSE (objective bug) = raw `{first:$N, rest:$M}`
  exposing the `[[SEP,item],…]` envelope. If `SEP` is an inline
  alternation `( a | b )` feeding a bare positional `$N`, it
  additionally triggers the emit-time `<invalid_sequence_access>`
  corruption (systemic class) — HIGH priority.
- Category B — op-chain / payload-per-iteration: `next ( OP next )*`
  where `OP` is a meaningful operator the consumer needs. Correct =
  `OP` lifted into a NAMED op-rule + bare `rest:$2` (the
  `systemverilog.ebnf` `binary_operator` idiom). The systemic binop
  instances are already fixed this session (RTL-CE-0001, SVPP-0001,
  RTL-FE-0001, VHDL-0001) → Cat-B-resolved, correct. Residual Cat-B
  bug = any OTHER rule still using an inline `( a | b )` iteration lead
  feeding bare positional `$N` (not one of the fixed binop levels).
- Category C — `X X*` bare repetition (no separator, no per-iteration
  multi-positional payload). `{first:$1, rest:$2}` is FINE (`$2` is
  already a clean array of `X`). Not a bug.

## Scope and excluded-grammar justification

In scope (product grammars): `grammars/regex.ebnf`,
`grammars/systemverilog.ebnf`, `grammars/systemverilog_preprocessor.ebnf`,
`grammars/vhdl.ebnf`, `grammars/rtl_frontend.ebnf`,
`grammars/rtl_const_expr.ebnf`.

(The pre-scope/task text refers to this grammar as
"systemverilog_preprocessor"; the actual filename is
`grammars/systemverilog_preprocessor.ebnf` — same surface.)

| Excluded grammar | `first/lhs/rest:$N` | Reason excluded (verified) |
| --- | --- | --- |
| `return_annotation.ebnf` | 0 | Annotation meta-grammar (the `->` language itself), not a typed product surface. |
| `semantic_annotation.ebnf` | 0 | Semantic-annotation meta-grammar; not a product AST surface. |
| `builtin_return_annotation.ebnf` | 0 | Behavioral spec of the bootstrap return parser; not a typed product. |
| `builtin_semantic_annotation.ebnf` | 0 | Behavioral spec of the bootstrap semantic parser; not a typed product. |
| `ebnf.ebnf` | 0 | EBNF bootstrap meta-grammar (self-description); not a product. |
| `json.ebnf` | 0 | JSON bootstrap grammar; not an in-campaign typed product surface. |
| `systemverilog_2017_lrm_extracted.ebnf` | 0 | Raw IEEE-1800-2017 LRM extraction snapshot, untyped (no `->` AST shaping). |
| `systemverilog_2023_lrm_extracted.ebnf` | 0 | Raw IEEE-1800-2023 LRM extraction snapshot, untyped. |
| `systemverilog_lrm_profiled_generated.ebnf` | 0 | Auto-generated profile snapshot, untyped reference. |
| `systemverilog_lrm_profiled_wrapper.ebnf` | 0 | Profile-aware wrapper shell, untyped. |
| `verilog_2005_lrm_extracted.ebnf` | 0 | Raw Verilog-2005 LRM extraction snapshot, untyped. |

Each excluded grammar was verified to contain zero
`{first/lhs/rest:$N}` occurrences and to be either an
annotation/bootstrap meta-grammar or an untyped LRM-extracted
reference snapshot — none is a typed product AST surface.

## regex.ebnf

In scope but zero occurrences. `regex.ebnf` uses `=` rules with the
`@generate` / `@semantic_value` annotation idiom and flat-emit
`[prefix*, last]` extraction; it has no `{first/lhs/rest:$N}` envelopes
and no bare positional `$N` over a `( )*` / `( )+` iteration. Nothing
to classify. Matches pre-scope (0).

## rtl_const_expr.ebnf

| Rule | Grammar:Line | RHS shape (brief) | Category | Objective bug? | Priority | Probe evidence needed | Recommended fix |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `logical_or_expr` | rtl_const_expr.ebnf:19-20 | `logical_and_expr ( logical_or logical_and_expr )*`; lead = NAMED rule `logical_or` (l.80) | B (resolved) | no | none | static-conclusive | none — correct (named op-rule + bare `rest:$2`) |
| `logical_and_expr` | rtl_const_expr.ebnf:21-22 | `bit_or_expr ( logical_and bit_or_expr )*`; NAMED `logical_and` (l.81) | B (resolved) | no | none | static-conclusive | none — correct |
| `bit_or_expr` | rtl_const_expr.ebnf:23-24 | `bit_xor_expr ( bit_or bit_xor_expr )*`; NAMED `bit_or` (l.82) | B (resolved) | no | none | static-conclusive | none — correct |
| `bit_xor_expr` | rtl_const_expr.ebnf:25-26 | `bit_and_expr ( bit_xor bit_and_expr )*`; NAMED `bit_xor` (l.83) | B (resolved) | no | none | static-conclusive | none — correct |
| `bit_and_expr` | rtl_const_expr.ebnf:27-28 | `equality_expr ( bit_and equality_expr )*`; NAMED `bit_and` (l.84) | B (resolved) | no | none | static-conclusive | none — correct |
| `equality_expr` | rtl_const_expr.ebnf:29-30 | `relational_expr ( equality_op relational_expr )*`; NAMED `equality_op` (l.40) | B (resolved) | no | none | static-conclusive | none — correct |
| `relational_expr` | rtl_const_expr.ebnf:31-32 | `shift_expr ( relational_op shift_expr )*`; NAMED `relational_op` (l.41) | B (resolved) | no | none | static-conclusive | none — correct |
| `shift_expr` | rtl_const_expr.ebnf:33-34 | `additive_expr ( shift_op additive_expr )*`; NAMED `shift_op` (l.42) | B (resolved) | no | none | static-conclusive | none — correct |
| `additive_expr` | rtl_const_expr.ebnf:35-36 | `multiplicative_expr ( additive_op multiplicative_expr )*`; NAMED `additive_op` (l.43) | B (resolved) | no | none | static-conclusive | none — correct |
| `multiplicative_expr` | rtl_const_expr.ebnf:37-38 | `unary_expr ( multiplicative_op unary_expr )*`; NAMED `multiplicative_op` (l.44) | B (resolved) | no | none | static-conclusive | none — correct |

All 10 `binop_chain` rules confirmed Cat-B-resolved (RTL-CE-0001 named
op-rule idiom). No bug. No churn.

## systemverilog_preprocessor.ebnf

| Rule | Grammar:Line | RHS shape (brief) | Category | Objective bug? | Priority | Probe evidence needed | Recommended fix |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `macro_formals` | systemverilog_preprocessor.ebnf:104-105 (now 110-111) | `lparen macro_formal ( comma macro_formal )* rparen` → `{first:$2, rest:$3}`; SEP = single token `comma` | A | yes | med | probe `\`define M(a, b, c)` and inspect `macro_formals` node — confirm `rest` is raw `[[comma,macro_formal],…]` envelope | `[$2, $3::2*]` (drop comma; clean macro_formal list) |

**RESOLVED — FIXED (`PGEN-POST-SV-AUDIT-0002`, leaf POST-SV-AUDIT.2.1,
2026-05-17).** Original classification above kept verbatim as history.
`macro_formals` was changed in `grammars/systemverilog_preprocessor.ebnf`
from `{first:$2, rest:$3}` (raw-envelope misuse) to the canonical
extraction-spread `[$2, $3::2*]` (now at grammar lines 110-111). Fixed
in sv_preprocessor **parser release `1.0.3` / contract `1.0.3` /
AST-dump schema `3`** (schema `2 → 3`; consumer-visible shape change).
Real captured before→after for input `` `define M(a, b, c) a+b+c ``
(verified via `parseability_probe`), at `pp_define.formals`:
- BEFORE (≤ schema 2 / release 1.0.2):
  `{"first": {"default": [], "name": [[], "a"]}, "rest": [[[[], ","], {"default": [], "name": [[" "], "b"]}], [[[], ","], {"default": [], "name": [[" "], "c"]}]]}`
- AFTER (schema 3 / release 1.0.3):
  `[{"default": [], "name": [[], "a"]}, {"default": [], "name": [[" "], "b"]}, {"default": [], "name": [[" "], "c"]}]`
  — a clean flat list of `macro_formal` `{name, default}` objects.
Surface counts UNCHANGED 66 annotations / 28 distinct rules — only
`macro_formals`'s `annotation_type` changed `return_object` →
`return_array` and `normalized_text` `{first:$2, rest:$3}` →
`[$2, $3::2*]` (surface now 65 `return_object` + 1 `return_array`). No
`<invalid_sequence_access>` (clean Category-A shape improvement, NOT the
inline-alt corruption class). This is **not** a released-parser bug — it
is **not** logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
(that ledger is reserved for the `<invalid_sequence_access>`
corruption/crash class — SVPP-0001 et al.); it is tracked via this
ledger + the schema-`3` Schema-Versioning row +
`docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
"AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT)" + the sv_preprocessor
parser book. Manifest
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
re-locked (`macro_formals` `return_array`/`[$2, $3::2*]`, new
`macro_with_formals` sample); `systemverilog_preprocessor_ast_shape_contract`
passes.

(The session SVPP-0001 binop fix was in `sv_pp_const_expr`; this
`macro_formals` rule is a separate, still-raw pure-comma list — NOT a
binop, not covered by the closed inline-alt class. It is plain
raw-separator-envelope, no `<invalid_sequence_access>` risk. **Now
resolved — see RESOLVED note above.**)

## rtl_frontend.ebnf

| Rule | Grammar:Line | RHS shape (brief) | Category | Objective bug? | Priority | Probe evidence needed | Recommended fix |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `parameter_declaration_sequence` | rtl_frontend.ebnf:79-80 | `group ( comma group )*` → `{first:$1, rest:$2}`; SEP token `comma` | A | yes | med | probe a 2-param decl; confirm `rest` raw `[[comma,group],…]` | `[$1, $2::2*]` |
| `port_list` | rtl_frontend.ebnf:98-99 | `port_group ( comma port_group )*` → `{first:$1, rest:$2}`; SEP `comma` | A | yes | med | probe 2-port module; confirm raw envelope | `[$1, $2::2*]` |
| `port_group` | rtl_frontend.ebnf:101-102 | `dir dt? pr? port_item ( comma !port_direction_token port_item )*` → `{…, first:$4, rest:$5}`; iter = `( comma <neg-LA> port_item )*` single payload `port_item` after comma | A | yes | med | probe `input a, b`; confirm `rest` raw `[[comma,port_item],…]` (neg-LA emits nothing) | `[$4, $5::N*]` (parent to confirm extraction index past the `!` lookahead) |
| `genvar_declaration` | rtl_frontend.ebnf:120-121 | `kw_genvar identifier ( comma identifier )* semi` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `genvar i, j;`; confirm raw envelope | `[$2, $3::2*]` |
| `module_instantiation` | rtl_frontend.ebnf:123-124 | `… instance_item ( comma instance_item )* semi` → `{…, first:$3, rest:$4}`; SEP `comma` | A | yes | med | probe 2-instance line; confirm raw envelope | `[$3, $4::2*]` |
| `parameter_override_list` (named) | rtl_frontend.ebnf:129 | `&dot override ( comma &dot override )*` → `{kind:"named", first:$2, rest:$3}`; SEP `comma` (`&dot` is zero-width LA) | A | yes | med | probe `#(.A(1),.B(2))`; confirm raw envelope | keep `kind`, `[$2, $3::2*]` |
| `parameter_override_list` (positional) | rtl_frontend.ebnf:130 | `!dot override ( comma !dot override )*` → `{kind:"positional", first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `#(1,2)`; confirm raw envelope | keep `kind`, `[$2, $3::2*]` |
| `port_connection_list` (named) | rtl_frontend.ebnf:136 | `&dot conn ( comma &dot conn )*` → `{kind:"named", first:$2, rest:$3}` | A | yes | med | probe `.a(x),.b(y)`; confirm raw envelope | keep `kind`, `[$2, $3::2*]` |
| `port_connection_list` (positional) | rtl_frontend.ebnf:137 | `!dot conn ( comma !dot conn )*` → `{kind:"positional", first:$2, rest:$3}` | A | yes | med | probe `x, y`; confirm raw envelope | keep `kind`, `[$2, $3::2*]` |
| `net_declaration` | rtl_frontend.ebnf:144-145 | `dt pr? net_item ( comma net_item )* semi` → `{…, first:$3, rest:$4}`; SEP `comma` | A | yes | med | probe `logic a, b;`; confirm raw envelope | `[$3, $4::2*]` |
| `event_control_list` | rtl_frontend.ebnf:162-163 | `at lparen event_control_item ( ( comma \| kw_or ) event_control_item )* rparen` → `{first:$3, rest:$4}`; iter lead = INLINE ALT `( comma \| kw_or )` feeding bare `$4` | A (+ inline-alt-$N) | yes | HIGH | probe `always_ff @(posedge clk or negedge rst)` AND `@(a, b)` — confirm `<invalid_sequence_access>` in `rest` (inline-alt-$N corruption), not just raw envelope | lift `( comma \| kw_or )` → NAMED rule `event_separator := comma {kind:"comma"} \| kw_or {kind:"or"}`, then `event_control_list := at lparen event_control_item ( event_separator event_control_item )* rparen -> [$3, $4::2*]` (separator semantically irrelevant → drop it) |
| `assignment_target` (concat) | rtl_frontend.ebnf:189 | `lbrace target ( comma target )+ rbrace` → `{kind:"concat", first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `{a, b} = c;`; confirm raw envelope | keep `kind`, `[$2, $3::2*]` |
| `logical_or_expr` | rtl_frontend.ebnf:200-201 | `( logical_or … )*`; NAMED `logical_or` | B (resolved) | no | none | static-conclusive | none — correct (RTL-FE-0001) |
| `logical_and_expr` | rtl_frontend.ebnf:202-203 | NAMED `logical_and` | B (resolved) | no | none | static-conclusive | none — correct |
| `bit_or_expr` | rtl_frontend.ebnf:204-205 | NAMED `bit_or` | B (resolved) | no | none | static-conclusive | none — correct |
| `bit_xor_expr` | rtl_frontend.ebnf:206-207 | NAMED `bit_xor` | B (resolved) | no | none | static-conclusive | none — correct |
| `bit_and_expr` | rtl_frontend.ebnf:208-209 | NAMED `bit_and` | B (resolved) | no | none | static-conclusive | none — correct |
| `equality_expr` | rtl_frontend.ebnf:218-219 | NAMED `equality_op` (l.229) | B (resolved) | no | none | static-conclusive | none — correct |
| `relational_expr` | rtl_frontend.ebnf:220-221 | NAMED `relational_op` (l.230) | B (resolved) | no | none | static-conclusive | none — correct |
| `shift_expr` | rtl_frontend.ebnf:222-223 | NAMED `shift_op` (l.231) | B (resolved) | no | none | static-conclusive | none — correct |
| `additive_expr` | rtl_frontend.ebnf:224-225 | NAMED `additive_op` (l.232) | B (resolved) | no | none | static-conclusive | none — correct |
| `multiplicative_expr` | rtl_frontend.ebnf:226-227 | NAMED `multiplicative_op` (l.233) | B (resolved) | no | none | static-conclusive | none — correct |
| `repetition_expr` | rtl_frontend.ebnf:249-250 | `lbrace rtl_expr lbrace rtl_expr ( comma rtl_expr )* rbrace rbrace` → `{count:$2, first:$4, rest:$5}`; SEP `comma` | A | yes | med | probe `{3{a, b}}`; confirm raw envelope | keep `count`, `[$4, $5::2*]` |
| `concatenation_expr` | rtl_frontend.ebnf:252-253 | `lbrace rtl_expr ( comma rtl_expr )+ rbrace` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `{a, b, c}`; confirm raw envelope | `[$2, $3::2*]` |
| `scoped_identifier` | rtl_frontend.ebnf:261-262 | `identifier ( scope_resolution identifier )*` → `{first:$1, rest:$2}`; SEP `scope_resolution` (`::`) | A | yes | med | probe `pkg::sub::name`; confirm raw envelope | `[$1, $2::2*]` |
| `enum_type` | rtl_frontend.ebnf:289-290 | `kw_enum (base)? pr? lbrace enum_item ( comma enum_item )* ( comma )? rbrace` → `{…, first:$5, rest:$6}`; SEP `comma` | A | yes | med | probe `enum {A, B}`; confirm raw envelope | keep base/packed_range, `[$5, $6::2*]` |
| `struct_union_field` | rtl_frontend.ebnf:305-306 | `dt pr? name ( comma name )* semi` → `{…, first:$3, rest:$4}`; SEP `comma` | A | yes | med | probe `logic a, b;` field; confirm raw envelope | keep dt/pr, `[$3, $4::2*]` |

**RESOLVED — FIXED (`PGEN-POST-SV-AUDIT-0003`, leaf POST-SV-AUDIT.2.2,
2026-05-17).** The original classifications above are kept verbatim as
history. All **15 rtl_frontend Category-A raw-envelope list rules** plus
the `event_control_list` inline-alternation-`$N` corruption were fixed in
`grammars/rtl_frontend.ebnf`, the parser regenerated, and the manifest
`rust/test_data/ast_shape_contract/rtl_frontend_v1.json` re-locked
(`rtl_frontend_ast_shape_contract` passes). Fixed in **rtl_frontend
parser release `1.0.3` / contract `1.0.3` / AST-dump schema `3`**
(schema `2 → 3`; consumer-visible shape change). Parent
probe-verified all: zero `[[], ","]` separator-envelope leaks remain,
`invalid_sequence_access count: 0`, every edge-case extraction index
correct.

- The **15 Category-A** rules — old `{first/…,rest}` → new
  extraction-spread (parent probe-confirmed; the lookahead-bearing rules
  carry the proven positional offset):
  - `parameter_declaration_sequence` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `port_list` `{first:$1,rest:$2}` → `[$1, $2::2*]` (RESOLVED — FIXED)
  - `port_group` `{direction:$1,data_type:$2,packed_range:$3,first:$4,rest:$5}`
    → `{direction:$1,data_type:$2,packed_range:$3,ports:[$4, $5::3*]}`
    (the `!port_direction_token` negative-lookahead occupies a positional
    slot — probe-confirmed) (RESOLVED — FIXED)
  - `genvar_declaration` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `module_instantiation` `{module_name:$1,parameters:$2,first:$3,rest:$4}`
    → `{module_name:$1,parameters:$2,instances:[$3, $4::2*]}`
    (RESOLVED — FIXED)
  - `parameter_override_list` (named & positional)
    `{kind:…,first:$2,rest:$3}` → `{kind:…,items:[$2, $3::3*]}`
    (the `&dot`/`!dot` lookahead occupies a positional slot —
    probe-confirmed) (RESOLVED — FIXED)
  - `port_connection_list` (named & positional)
    `{kind:…,first:$2,rest:$3}` → `{kind:…,items:[$2, $3::3*]}`
    (the `&dot`/`!dot` lookahead occupies a positional slot —
    probe-confirmed) (RESOLVED — FIXED)
  - `net_declaration` `{data_type:$1,packed_range:$2,first:$3,rest:$4}`
    → `{data_type:$1,packed_range:$2,items:[$3, $4::2*]}`
    (RESOLVED — FIXED)
  - `assignment_target` concat `{kind:"concat",first:$2,rest:$3}`
    → `{kind:"concat",items:[$2, $3::2*]}` (RESOLVED — FIXED)
  - `repetition_expr` `{count:$2,first:$4,rest:$5}`
    → `{count:$2,items:[$4, $5::2*]}` (RESOLVED — FIXED)
  - `concatenation_expr` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `scoped_identifier` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `enum_type` `{base:$2,packed_range:$3,first:$5,rest:$6}`
    → `{base:$2,packed_range:$3,items:[$5, $6::2*]}` (RESOLVED — FIXED)
  - `struct_union_field` `{data_type:$1,packed_range:$2,first:$3,rest:$4}`
    → `{data_type:$1,packed_range:$2,names:[$3, $4::2*]}`
    (RESOLVED — FIXED)
- `event_control_list` (162-163, the HIGH-priority inline-alt-`$N`
  finding) — **RESOLVED — FIXED.** The inline `( comma | kw_or )`
  alternation was lifted into the new **un-annotated** named rule
  `event_separator := comma | kw_or` and `event_control_list` rewritten
  to `at lparen event_control_item ( event_separator event_control_item )* rparen -> [$3, $4::2*]`
  (separator dropped; clean `event_control_item` list). Probe-verified
  for `always_ff @(posedge clk or negedge rst) ;`:
  `procedural_block.event_control` is now a clean `[{edge,expr},…]` list,
  zero `<invalid_sequence_access>`.

The 15 Category-A corrections are a clean shape improvement and are
**not** logged in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (per the recorded
Decision in DEVELOPMENT_NOTES — that ledger is reserved for the
`<invalid_sequence_access>` corruption/crash class). The
`event_control_list` inline-alt-`$N` corruption **is** an
`<invalid_sequence_access>` defect and **is** logged there as the new
row `RTL-FE-0002` (status `Released`, fixed in rtl_frontend 1.0.3 /
schema 3). Annotation inventory **unchanged at 156 annotations / 74
distinct rules** (bare-list Cat-A rules flip `return_object` →
`return_array`; the `{…,items/ports/names:[…]}` ones stay
`return_object` with new `normalized_text`; `event_separator` is
un-annotated → not in the inventory; no count delta). Tracked via this
ledger + the schema-`3` Schema-Versioning row +
`docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md`
"AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT)" + the rtl_frontend
parser book.

## systemverilog.ebnf

| Rule | Grammar:Line | RHS shape (brief) | Category | Objective bug? | Priority | Probe evidence needed | Recommended fix |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `constant_expression` | systemverilog.ebnf:284-285 | `operand ( binary_operator attribute_instance* operand )* (…ternary)?` → `{first:$1, rest:$2, ternary:$3}`; lead = NAMED `binary_operator` (l.235) | B (resolved) | no | none | static-conclusive | none — correct |
| `expression_base` (operand_chain) | systemverilog.ebnf:296-297 | `operand ( binary_operator attribute_instance* operand )*` → `{kind:"operand_chain", first:$1, rest:$2}`; NAMED `binary_operator` | B (resolved) | no | none | static-conclusive | none — correct |
| `unsigned_number` | systemverilog.ebnf:345 | `decimal_digit ( kw_sv_rule_c82a06f6 \| decimal_digit )*` → `{first:$1, rest:$2}`; iter lead = INLINE ALT `( sv_rule-tok \| digit )` feeding bare `$2` | B (residual) / A+inline-alt | yes | HIGH | probe a multi-digit number through `unsigned_number`; confirm `rest` = `<invalid_sequence_access>` (inline-alt-$N) | lift `( kw_sv_rule_c82a06f6 \| decimal_digit )` → NAMED digit-or-sep rule, bare `rest:$2` (binary_operator idiom); separator `sv_rule` is a digit-group sep → likely `[$1, $2::… ]` after naming. Parent to confirm desired digit shape |
| `binary_value` | systemverilog.ebnf:544 | `binary_digit ( kw_sv_rule_c82a06f6 \| binary_digit )*` → `{first:$1, rest:$2}`; INLINE ALT lead | B (residual) / A+inline-alt | yes | HIGH | probe `4'b1010`; confirm `<invalid_sequence_access>` | same as `unsigned_number` (named lead-rule) |
| `hex_value` | systemverilog.ebnf:2062 | `hex_digit ( kw_sv_rule_c82a06f6 \| hex_digit )*` → `{first:$1, rest:$2}`; INLINE ALT lead | B (residual) / A+inline-alt | yes | HIGH | probe `8'hDEAD`; confirm `<invalid_sequence_access>` | same (named lead-rule) |
| `non_zero_unsigned_number` | systemverilog.ebnf:3064 | `non_zero_decimal_digit ( kw_sv_rule_c82a06f6 \| decimal_digit )*` → `{first:$1, rest:$2}`; INLINE ALT lead | B (residual) / A+inline-alt | yes | HIGH | probe a multi-digit nonzero number; confirm `<invalid_sequence_access>` | same (named lead-rule) |
| `octal_value` | systemverilog.ebnf:3099 | `octal_digit ( kw_sv_rule_c82a06f6 \| octal_digit )*` → `{first:$1, rest:$2}`; INLINE ALT lead | B (residual) / A+inline-alt | yes | HIGH | probe `3'o17`; confirm `<invalid_sequence_access>` | same (named lead-rule) |
| `case_generate_construct` | systemverilog.ebnf:693-694 | `… item item* kw_endcase` → `{expr:$3, items:{first:$5, rest:$6}}`; bare repeat `item item*` | C | no | none | static-conclusive | none — Cat C benign |
| `case_statement` | systemverilog.ebnf:733-734 | `… case_item case_item* kw_endcase` → `{…, items:{first:$6, rest:$7}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `combinational_body` | systemverilog.ebnf:1028-1029 | `kw_table entry entry* kw_endtable` → `{entries:{first:$2, rest:$3}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `data_type` (struct_union) | systemverilog.ebnf:1501 | `… lbrace member member* rbrace …` → `{…, members:{first:$4, rest:$5}, …}`; bare repeat `member member*` | C | no | none | static-conclusive | none — Cat C benign |
| `delay_sv_2017` (triple_optional) | systemverilog.ebnf:1589 | `hash lparen mtm ( comma mtm ( comma mtm )? )? rparen` → `{kind:"triple_optional", first:$3, rest:$4}`; `$4` references an OPTIONAL `?` group, NOT a `*`/`+` iteration | n/a (not an iteration) | no | none | static-conclusive | none — out of taxonomy (optional group, fixed arity ≤3); style only, do not churn |
| `delay_sv_2023` (triple_optional) | systemverilog.ebnf:1597 | same as 1589 (sv_2023 profile) | n/a (not an iteration) | no | none | static-conclusive | none — out of taxonomy |
| `let_list_of_arguments` (named_only) | systemverilog.ebnf:2357-2358 | `dot id lparen (arg)? rparen ( comma dot id lparen (arg)? rparen )*` → `{kind:"named_only", first_name:$2, first_value:$4, rest:$6}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe `.a(1), .b(2)` named let args; confirm `rest` raw `[[comma,dot,id,lparen,arg,rparen],…]` | factor `( comma dot id lparen (arg)? rparen )` unit into a named rule emitting `{name,value}` then `[…, $6::*]`; parent to confirm extraction shape |
| `list_of_arguments_mixed_head` (chain) | systemverilog.ebnf:2409-2410 | `( expression )? comma list_of_arguments_mixed_head` → `{kind:"chain", expr:$1, rest:$3}`; recursion, `$3` is the recursive tail (single node), NOT a `*`/`+` iter | C-like (not an iteration) | no | none | static-conclusive | none — `$3` is a single recursive-rule node, not an iteration envelope; benign |
| `list_of_interface_identifiers` | systemverilog.ebnf:2435-2436 | `iface_id udim* ( comma iface_id udim* )*` → `{first:{name:$1,dims:$2}, rest:$3}`; SEP `comma`, multi-field per iter (`iface_id` + `udim*`) | A (structured) | yes | med | probe `a [1:0], b` interface list; confirm `rest` raw `[[comma,iface_id,[udim…]],…]` | factor `( comma iface_id udim* )` into a named `{name,dims}` rule + `[{name:$1,dims:$2}, $3::*]`; parent to confirm. (Sibling `list_of_genvar_identifiers` l.2432 uses the correct `[$1,$2::2*]` — that pattern is the model for single-element lists) |
| `list_of_port_identifiers` | systemverilog.ebnf:2490-2491 | `port_id udim* ( comma port_id udim* )*` → `{first:{name:$1,dims:$2}, rest:$3}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe `a, b [3:0]` port-id list; confirm raw envelope | factor named `{name,dims}` unit + extraction-spread; parent to confirm |
| `list_of_tf_variable_identifiers` | systemverilog.ebnf:2499-2500 | `port_id vdim* (assign expr)? ( comma port_id vdim* (assign expr)? )*` → `{first:{name:$1,dims:$2,init:$3}, rest:$4}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe `a = 1, b` tf-var list; confirm raw envelope | factor named `{name,dims,init}` unit + extraction-spread; parent to confirm |
| `list_of_variable_identifiers` | systemverilog.ebnf:2514-2515 | `var_id vdim* ( comma var_id vdim* )*` → `{first:{name:$1,dims:$2}, rest:$3}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe `a, b [1:0]` var-id list; confirm raw envelope | factor named `{name,dims}` unit + extraction-spread; parent to confirm |
| `list_of_variable_port_identifiers` | systemverilog.ebnf:2517-2518 | `port_id vdim* (assign cexpr)? ( comma port_id vdim* (assign cexpr)? )*` → `{first:{name:$1,dims:$2,init:$3}, rest:$4}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe `a = 0, b` var-port list; confirm raw envelope | factor named `{name,dims,init}` unit + extraction-spread; parent to confirm |
| `module_path_expression` (chain) | systemverilog.ebnf:2802-2803 | `operand ( binary_module_path_operator attribute_instance* operand )*` → `{kind:"chain", first:$1, rest:$2}`; lead = NAMED `binary_module_path_operator` (l.542-ish) | B (resolved) | no | none | static-conclusive | none — correct (named op-rule) |
| `net_alias` | systemverilog.ebnf:2889-2890 | `kw_alias lvalue assign lvalue ( assign lvalue )* semi` → `{first:$2, second:$4, rest:$5}`; SEP single token `assign` (`=`) | A | yes | med | probe `alias a = b = c;`; confirm `rest` raw `[[assign,lvalue],…]` | keep `first`/`second`, `[$2, $4, $5::2*]` (or `rest:[$5::2*]`); parent to confirm preferred merge of first/second into the list |
| `parameter_port_list` (type_only) | systemverilog.ebnf:3327-3328 | `hash lparen kw_type ta ( comma kw_type ta )* rparen` → `{kind:"type_only", first:$4, rest:$5}`; SEP `comma`, per-iter payload = `kw_type type_assignment` (2 tokens) | A (structured) | yes | med | probe `#(type T=int, type U=bit)`; confirm raw `[[comma,kw_type,ta],…]` | drop comma; either extract `type_assignment` (`$5::3*`) or factor named unit; parent to confirm (sibling `declarations` branch l.3330 uses correct `[$3, $4::2*]`) |
| `assignment_pattern` (named, 3386) | systemverilog.ebnf:3385-3386 | `tick lbrace mid colon pattern ( comma mid colon pattern )* rbrace` → `{kind:"named", entries:{first:{name:$3,pattern:$5}, rest:$6}}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe `'{a:1, b:2}`; confirm `rest` raw `[[comma,mid,colon,pattern],…]` | factor `( comma mid colon pattern )` → named `{name,pattern}` rule + extraction-spread; parent to confirm (sibling `ordered` branch uses correct `[$3,$4::2*]`) |
| `assignment_pattern` (named, 3402) | systemverilog.ebnf:3401-3402 | second profile copy of above | A (structured) | yes | med | same as 3386 | same fix |
| `property_list_of_arguments` (named_only) | systemverilog.ebnf:3800-3801 | `dot id lparen (arg)? rparen ( comma dot id lparen (arg)? rparen )*` → `{kind:"named_only", first_name:$2, first_value:$4, rest:$6}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe a 2-named-arg property call; confirm raw envelope | factor named `{name,value}` unit + extraction-spread; parent to confirm |
| `randcase_statement` | systemverilog.ebnf:3925-3926 | `kw_randcase item item* kw_endcase` → `{items:{first:$2, rest:$3}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `randsequence_statement_sv_2017` | systemverilog.ebnf:3935-3936 | `… production production* kw_endsequence` → `{…, productions:{first:$5, rest:$6}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `randsequence_statement_sv_2023` | systemverilog.ebnf:3939-3940 | same, sv_2023 profile; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `rs_case` | systemverilog.ebnf:3973-3974 | `kw_case … rs_case_item rs_case_item* kw_endcase` → `{expr:$3, items:{first:$5, rest:$6}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `rs_production_list_sv_2017` (productions) | systemverilog.ebnf:4042-4043 | `rs_prod rs_prod*` → `{kind:"productions", items:{first:$1, rest:$2}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `rs_production_list_sv_2017` (rand_join) | systemverilog.ebnf:4044-4045 | `kw_rand kw_join (…)? pi pi pi*` → `{…, items:{first:$4, second:$5, rest:$6}}`; bare repeat of `production_item` after two fixed | C | no | none | static-conclusive | none — Cat C benign (`$6` = clean `production_item*`) |
| `rs_production_list_sv_2023` (productions) | systemverilog.ebnf:4048-4049 | `rs_prod rs_prod*` → `{kind:"productions", items:{first:$1, rest:$2}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `rs_production_list_sv_2023` (rand_join) | systemverilog.ebnf:4050-4051 | bare repeat of item after two fixed | C | no | none | static-conclusive | none — Cat C benign |
| `sequence_list_of_arguments` (named_only) | systemverilog.ebnf:4197-4198 | `dot id lparen (arg)? rparen ( comma dot id lparen (arg)? rparen )*` → `{kind:"named_only", first_name:$2, first_value:$4, rest:$6}`; SEP `comma`, multi-field per iter | A (structured) | yes | med | probe a 2-named-arg sequence call; confirm raw envelope | factor named `{name,value}` unit + extraction-spread; parent to confirm |
| `sequential_body` | systemverilog.ebnf:4217-4218 | `(udp_init)? kw_table entry entry* kw_endtable` → `{initial:$1, entries:{first:$3, rest:$4}}`; bare repeat | C | no | none | static-conclusive | none — Cat C benign |
| `udp_declaration_sv_2017` (nonansi) | systemverilog.ebnf:4789-4790 | `udp_nonansi pd pd* udp_body kw_endprimitive (…)?` → `{…, port_decls:{first:$2, rest:$3}, …}`; bare repeat `pd pd*` | C | no | none | static-conclusive | none — Cat C benign |
| `udp_declaration_sv_2023` (nonansi) | systemverilog.ebnf:4801-4802 | same, sv_2023 profile; bare repeat | C | no | none | static-conclusive | none — Cat C benign |

**RESOLVED — FIXED (`PGEN-POST-SV-AUDIT-0005`, leaf POST-SV-AUDIT.2.4a,
2026-05-17).** The original classifications above are kept verbatim as
history. Two systemverilog worklist items were dispositioned and fixed
in `grammars/systemverilog.ebnf`; the parser was regenerated and the
manifest `rust/test_data/ast_shape_contract/systemverilog_v1.json`
re-locked (new `net_alias` sample + `calibration_history` entry #117;
`systemverilog_ast_shape_contract` passes). Fixed in **systemverilog
parser release `1.0.116` / contract `1.0.116` / AST-dump schema `2`**
(schema `1 → 2`; the reachable `net_alias` consumer-visible shape change
drives the bump).

- **`net_alias` (2889-2890) — RESOLVED — FIXED (Category-A,
  reachable, consumer-visible).** Old `{first: $2, second: $4, rest: $5}`
  (raw `[[assign, net_lvalue], …]` single-token-`assign`-separator
  `rest` envelope) → new `{lvalues: [$2, $4, $5::2*]}` (clean flat
  `net_lvalue[]` list of **all** aliased lvalues; `=` separators
  dropped). Stays `return_object` (object with one array field), new
  `normalized_text` only. Parent probe-verified on
  `module m; wire a, b, c; alias a = b = c; endmodule`:
  `net_alias` = `{"lvalues":[{…a…},{…b…},{…c…}]}`. Single-token
  separator, **no** inline alternation, **no**
  `<invalid_sequence_access>` — clean Category-A improvement, **NOT a
  bug-ledger row**.
- **5 number rules (`unsigned_number` 345, `binary_value` 544,
  `hex_value` 2062, `non_zero_unsigned_number` 3064, `octal_value`
  3099) — RESOLVED — FIXED-DEFENSIVE.** Each was
  `<digit> ( kw_sv_rule_c82a06f6 | <digit> )* -> {first: $1, rest: $2}`
  — an inline alternation as the `( … )*` iteration lead feeding bare
  `$2` (the systemic inline-alternation-`$N` corruption class). Fixed
  by lifting the inline alternation into a new **un-annotated** named
  tail rule (`unsigned_number_tail := kw_sv_rule_c82a06f6 | decimal_digit`,
  etc.) so the iteration becomes `( <rule>_tail )*` and `$2` binds
  cleanly; the `{first: $1, rest: $2}` annotation **text is
  unchanged**. This is the identical transformation empirically proven
  6× this session (RTL-CE / SVPP-0001 / RTL-FE-0001 / VHDL-0001 /
  RTL-FE-0002) — correct by construction. **Honest disposition: the
  corruption is structurally present but NOT consumer-reproducible.**
  The parent empirically established that the SV `systemverilog_file`
  root **rejects every numeric-bearing top-level construct**
  (`parameter` / `localparam` / `assign` / `$display` / packed ranges
  `[15:0]` / module-parameter headers) in **all** profiles
  (`default` / `sv_2017` / `sv_2023`); only minimal constructs
  (`module m; endmodule`,
  `module m; wire a, b, c; alias a = b = c; endmodule`) parse. A
  multi-digit number is therefore unreachable via valid `source_text`
  (a pre-existing SV-grammar-root coverage limitation, **separate from
  this defect and explicitly out of POST-SV-AUDIT scope**). This is a
  **defensive structural correction**, **NOT** a
  `PGEN_RELEASED_PARSER_BUG_LEDGER` row — we do not claim a released
  defect that no valid input can trigger; this is the honest
  disposition. `kw_sv_rule_c82a06f6 := trivia /sv_rule\b/` is itself a
  degenerate LRM-extraction artifact (a separate grammar-quality
  matter, out of scope here — noted as observation only).

Counts: annotation **2290** (UNCHANGED — `net_alias` text-only
`normalized_text` change; the 5 number rules' annotation text is
unchanged; the 5 `*_tail` rules are un-annotated and not in the
inventory — **no count delta**), **999** distinct annotated rules
(UNCHANGED). Same accept set. **No bug-ledger row** (neither item is
a consumer-reproducible released `<invalid_sequence_access>` defect).
The **11 structured-per-iteration Cat-A SV rules remain OPEN**
(POST-SV-AUDIT.2.4b — *not* touched/closed here).

## vhdl.ebnf

| Rule | Grammar:Line | RHS shape (brief) | Category | Objective bug? | Priority | Probe evidence needed | Recommended fix |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `library_clause` | vhdl.ebnf:29-30 | `kw_library identifier ( comma identifier )* semi` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `library a, b;`; confirm raw envelope | `[$2, $3::2*]` |
| `use_clause` | vhdl.ebnf:31-32 | `kw_use selected_name ( comma selected_name )* semi` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `use a, b;`; confirm raw envelope | `[$2, $3::2*]` |
| `selected_name` | vhdl.ebnf:42-43 | `identifier ( dot identifier )*` → `{first:$1, rest:$2}`; SEP `dot` | A | yes | med | probe `a.b.c`; confirm raw envelope | `[$1, $2::2*]` |
| `identifier_list` | vhdl.ebnf:44-45 | `identifier ( comma identifier )*` → `{first:$1, rest:$2}`; SEP `comma` | A | yes | med | probe `a, b, c`; confirm raw envelope | `[$1, $2::2*]` |
| `generic_interface_list` | vhdl.ebnf:62-63 | `gie ( semi gie )*` → `{first:$1, rest:$2}`; SEP `semi` | A | yes | med | probe 2-generic clause; confirm raw envelope | `[$1, $2::2*]` |
| `port_interface_list` | vhdl.ebnf:73-74 | `pie ( semi pie )*` → `{first:$1, rest:$2}`; SEP `semi` | A | yes | med | probe 2-port clause; confirm raw envelope | `[$1, $2::2*]` |
| `parameter_list` | vhdl.ebnf:171-172 | `lparen pie ( semi pie )* rparen` → `{first:$2, rest:$3}`; SEP `semi` | A | yes | med | probe `(a: in t; b: in t)`; confirm raw envelope | `[$2, $3::2*]` |
| `enumeration_type_definition` | vhdl.ebnf:203-204 | `lparen identifier ( comma identifier )* rparen` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `(A, B, C)`; confirm raw envelope | `[$2, $3::2*]` |
| `index_constraint` | vhdl.ebnf:218-219 | `lparen discrete_range ( comma discrete_range )* rparen` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `(0 to 1, 2 to 3)`; confirm raw envelope | `[$2, $3::2*]` |
| `target` (aggregate) | vhdl.ebnf:244 | `lparen target ( comma target )+ rparen` → `{kind:"aggregate", first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `(a, b) <= c;`; confirm raw envelope | keep `kind`, `[$2, $3::2*]` |
| `association_list` | vhdl.ebnf:253-254 | `lparen assoc ( comma assoc )* rparen` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `map (a => 1, b => 2)`; confirm raw envelope | `[$2, $3::2*]` |
| `sensitivity_list` | vhdl.ebnf:276-277 | `selected_name ( comma selected_name )*` → `{first:$1, rest:$2}`; SEP `comma` | A | yes | med | probe `process (clk, rst)`; confirm raw envelope | `[$1, $2::2*]` |
| `actual_parameter_part` | vhdl.ebnf:309-310 | `lparen ape ( comma ape )* rparen` → `{first:$2, rest:$3}`; SEP `comma` | A | yes | med | probe `f(a, b)`; confirm raw envelope | `[$2, $3::2*]` |
| `choices` | vhdl.ebnf:323-324 | `choice ( bar choice )*` → `{first:$1, rest:$2}`; SEP `bar` (`\|`) | A | yes | med | probe `when 1 \| 2 =>`; confirm raw envelope | `[$1, $2::2*]` |
| `aggregate` (named_first) | vhdl.ebnf:379 | `lparen acl arrow expr ( comma aea )* rparen` → `{kind:"named_first", first_choices:$2, first_value:$4, rest:$5}`; SEP `comma`, single elem `aea` | A | yes | med | probe `(others => 0, 1 => x)`; confirm `rest` raw `[[comma,aea],…]` | keep `kind`/`first_*`, `[$5::2*]` for rest |
| `aggregate` (positional_first) | vhdl.ebnf:380 | `lparen expr comma aea ( comma aea )* rparen` → `{kind:"positional_first", first_value:$2, second:$4, rest:$5}`; SEP `comma`, single elem `aea` | A | yes | med | probe `(a, b, c)` aggregate; confirm raw envelope | keep `kind`/`first_value`/`second`, `[$5::2*]` for rest |
| `aggregate_choice_list` | vhdl.ebnf:383-384 | `aggregate_choice ( bar aggregate_choice )*` → `{first:$1, rest:$2}`; SEP `bar` | A | yes | med | probe `a \| b \| c =>`; confirm raw envelope | `[$1, $2::2*]` |
| `expression` | vhdl.ebnf:348-349 | `relation ( logical_operator relation )*`; NAMED `logical_operator` (l.391) | B (resolved) | no | none | static-conclusive | none — correct (VHDL-0001) |
| `relation` | vhdl.ebnf:350-351 | `simple_expression ( relational_operator simple_expression )?`; NAMED `relational_operator` (l.398); note iter is `?` not `*` but lead is named so bare `rest:$2` is fine | B (resolved) | no | none | static-conclusive | none — correct |
| `simple_expression` | vhdl.ebnf:361-362 | `(plus\|minus)? term ( adding_operator term )*`; NAMED `adding_operator` (l.405); leading `(plus\|minus)?` sign is NOT an iter lead (documented unaffected) | B (resolved) | no | none | static-conclusive | none — correct |
| `term` | vhdl.ebnf:363-364 | `factor ( multiplying_operator factor )*`; NAMED `multiplying_operator` (l.409) | B (resolved) | no | none | static-conclusive | none — correct |
| `factor` | vhdl.ebnf:365-366 | `primary ( power primary )?`; NAMED single-token `power` (l.463) | B (resolved) | no | none | static-conclusive | none — correct |

**RESOLVED — FIXED (`PGEN-POST-SV-AUDIT-0004`, leaf POST-SV-AUDIT.2.3,
2026-05-17).** The original classifications above are kept verbatim as
history. All **17 vhdl Category-A raw-envelope list rules** were fixed
in `grammars/vhdl.ebnf`, the parser regenerated, and the manifest
`rust/test_data/ast_shape_contract/vhdl_v1.json` re-locked (new
`cat_a_shapes` sample; `vhdl_ast_shape_contract` passes). Fixed in
**vhdl parser release `1.0.3` / contract `1.0.3` / AST-dump schema `3`**
(schema `2 → 3`; consumer-visible shape change). Parent
probe-verified all: zero `[[],","]` / `[[],";"]` / `[[],"."]` /
`[[],"|"]` separator-envelope leaks, `invalid_sequence_access: 0`,
aggregate `rest` now a clean list, no leftover `{first, rest}` objects.
The 17 rules — old `{first/…,rest}` → new extraction-spread:

- The 14 bare-list rules — old `{first,rest}` (resp. `{…,first,rest}`)
  → new top-level `[$F, $R::2*]` (`return_object` → `return_array`):
  - `library_clause` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `use_clause` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `selected_name` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `identifier_list` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `generic_interface_list` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `port_interface_list` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `parameter_list` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `enumeration_type_definition` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `index_constraint` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `association_list` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `sensitivity_list` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `actual_parameter_part` `{first:$2,rest:$3}` → `[$2, $3::2*]`
    (RESOLVED — FIXED)
  - `choices` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
  - `aggregate_choice_list` `{first:$1,rest:$2}` → `[$1, $2::2*]`
    (RESOLVED — FIXED)
- `target` aggregate branch
  `{kind:"aggregate",first:$2,rest:$3}` →
  `{kind:"aggregate",items:[$2, $3::2*]}` (stays `return_object`, new
  `normalized_text`) (RESOLVED — FIXED)
- `aggregate` (both branches — kept the meaningful
  `first_choices`/`first_value`/`second` fields, only the trailing
  iteration `rest` cleaned):
  `{kind:"named_first",first_choices:$2,first_value:$4,rest:$5}` →
  `{kind:"named_first",first_choices:$2,first_value:$4,rest:[$5::2*]}`;
  `{kind:"positional_first",first_value:$2,second:$4,rest:$5}` →
  `{kind:"positional_first",first_value:$2,second:$4,rest:[$5::2*]}`
  (stays `return_object`, new `normalized_text`) (RESOLVED — FIXED)

All 17 are clean Category-A shape improvements — every separator is a
single token (comma / semi / dot / bar), there is **no** inline
alternation and **no** `<invalid_sequence_access>`. They are therefore
**not** logged in
`docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` (per the recorded
Decision in DEVELOPMENT_NOTES — that ledger is reserved for the
`<invalid_sequence_access>` corruption/crash class; `VHDL-0001`, the
systemic inline-alternation defect, lives there; this Category-A batch
adds **no** bug-ledger row, unlike POST-SV-AUDIT.2.2's `RTL-FE-0002`
which *was* corruption). Annotation inventory **unchanged at 256
annotations / 112 distinct rules** (the 14 bare-list Cat-A rules flip
`return_object` → `return_array`; the `target`-aggregate + `aggregate`
ones stay `return_object` with new `normalized_text`; no count delta).
Tracked via this ledger + the schema-`3` Schema-Versioning row +
`docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`
"AST-Shape Corrections — 1.0.3 (POST-SV-AUDIT)" + the vhdl parser book.

## Objective-bug worklist (for POST-SV-AUDIT.2)

### Static-conclusive objective bugs (no probe needed to confirm the class; probe only to capture the before/after artifact)

Pure single-token-separator `X ( SEP X )*` lists rendered as raw
`{first/lhs,rest}` envelopes — the separator carries no payload, the
consumer must walk past `[[SEP,item],…]`. Fix = extraction-spread
`[$N, $M::2*]` (keep any sibling fields like `kind`). These are
static-conclusive Cat-A misuse:

- systemverilog_preprocessor.ebnf: `macro_formals` (104-105). **DONE
  — fixed in sv_preprocessor 1.0.3 / schema 3, `PGEN-POST-SV-AUDIT-0002`
  (leaf POST-SV-AUDIT.2.1, 2026-05-17); `[$2, $3::2*]`. See the RESOLVED
  note in the systemverilog_preprocessor.ebnf section above.**
- rtl_frontend.ebnf: `parameter_declaration_sequence` (79-80),
  `port_list` (98-99), `port_group` (101-102, comma-after `!`
  lookahead — parent confirms index), `genvar_declaration` (120-121),
  `module_instantiation` (123-124),
  `parameter_override_list` named+positional (129, 130),
  `port_connection_list` named+positional (136, 137),
  `net_declaration` (144-145),
  `assignment_target` concat (189), `repetition_expr` (249-250),
  `concatenation_expr` (252-253), `scoped_identifier` (261-262),
  `enum_type` (289-290), `struct_union_field` (305-306).
  **DONE — all 15 fixed in rtl_frontend 1.0.3 / schema 3,
  `PGEN-POST-SV-AUDIT-0003` (leaf POST-SV-AUDIT.2.2, 2026-05-17);
  extraction-spreads `[$N, $M::K*]` (lookahead-bearing rules carry the
  proven positional offset). See the RESOLVED note in the
  rtl_frontend.ebnf section above.**
- vhdl.ebnf: `library_clause` (30), `use_clause` (32),
  `selected_name` (43), `identifier_list` (45),
  `generic_interface_list` (63), `port_interface_list` (74),
  `parameter_list` (172), `enumeration_type_definition` (204),
  `index_constraint` (219), `target` aggregate (244),
  `association_list` (254), `sensitivity_list` (277),
  `actual_parameter_part` (310), `choices` (324),
  `aggregate` named_first (379) + positional_first (380),
  `aggregate_choice_list` (384).
  **DONE — all 17 fixed in vhdl 1.0.3 / schema 3,
  `PGEN-POST-SV-AUDIT-0004` (leaf POST-SV-AUDIT.2.3, 2026-05-17);
  the 14 bare-list rules → top-level `[$F, $R::2*]`
  (`return_object` → `return_array`), the `target` aggregate branch →
  `{kind:"aggregate",items:[$2,$3::2*]}` and both `aggregate`
  branches → `rest:[$5::2*]` (stay `return_object`, new
  `normalized_text`). All single-token-separator Cat-A — NO
  `<invalid_sequence_access>`, NO bug-ledger row. 256/112 unchanged.
  See the RESOLVED note in the vhdl.ebnf section above.**
- systemverilog.ebnf: `net_alias` (2889-2890, SEP = single token
  `assign`). **DONE — fixed in systemverilog 1.0.116 / schema 2,
  `PGEN-POST-SV-AUDIT-0005` (leaf POST-SV-AUDIT.2.4a, 2026-05-17);
  `{first, second, rest}` → `{lvalues: [$2, $4, $5::2*]}` (clean flat
  list, stays `return_object`, new `normalized_text`). Reachable,
  consumer-visible, probe-verified — clean Category-A, NO bug-ledger
  row. See the RESOLVED note in the systemverilog.ebnf section above.**

### HIGH-priority — suspected inline-alternation-$N corruption (needs parent probe-confirmation of `<invalid_sequence_access>`)

These use an inline `( a | b )` alternation as the iteration lead
feeding a bare positional `$N` — the same systemic class as the
already-fixed binop levels, but NOT a binop level (so not covered by
the closed INLINE-ALT-FIX). High priority; fix = lift the
alternation into a NAMED rule (the `binary_operator` idiom), then a
clean `rest:$N` or extraction-spread.

- rtl_frontend.ebnf `event_control_list` (162-163):
  `( ( comma | kw_or ) event_control_item )*` → `{first:$3, rest:$4}`.
  This is the known concrete finding. Disposition: the
  `( comma | kw_or )` is a semantically-irrelevant SEPARATOR (both
  comma and `or` just join sensitivity entries) → Cat-A misuse with
  the inline-alt-$N corruption overlay. Recommended fix: lift
  `( comma | kw_or )` into a named rule
  `event_separator := comma -> {kind:"comma"} | kw_or -> {kind:"or"}`
  and rewrite to
  `at lparen event_control_item ( event_separator event_control_item )* rparen -> [$3, $4::2*]`
  (drop the separator entirely — consumers only want the item list).
  Probe inputs: `always_ff @(posedge clk or negedge rst) ;` and
  `always_ff @(a, b) ;` — confirm current `rest` emits
  `<invalid_sequence_access>` (corruption) vs merely a raw envelope.
  **DONE — fixed in rtl_frontend 1.0.3 / schema 3,
  `PGEN-POST-SV-AUDIT-0003` (leaf POST-SV-AUDIT.2.2, 2026-05-17).** The
  inline `( comma | kw_or )` was lifted into the new **un-annotated**
  named rule `event_separator := comma | kw_or` (the separator carries
  no payload a consumer needs, so it is dropped rather than `{kind}`-
  tagged) and `event_control_list` rewritten to
  `at lparen event_control_item ( event_separator event_control_item )* rparen -> [$3, $4::2*]`.
  Probe-verified for `always_ff @(posedge clk or negedge rst) ;`:
  `procedural_block.event_control` is now a clean `[{edge,expr},…]`
  list, zero `<invalid_sequence_access>`. This is the
  `<invalid_sequence_access>` corruption class, so it **is** logged in
  `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` as the new row
  `RTL-FE-0002` (status `Released`). See the RESOLVED note in the
  rtl_frontend.ebnf section above.
- systemverilog.ebnf number rules — all five share
  `digit ( kw_sv_rule_c82a06f6 | digit )*` → `{first:$1, rest:$2}`
  where `kw_sv_rule_c82a06f6 := trivia /sv_rule\b/` is a literal
  digit-group separator token used inline-alternated with the digit:
  - `unsigned_number` (345)
  - `binary_value` (544)
  - `hex_value` (2062)
  - `non_zero_unsigned_number` (3064)
  - `octal_value` (3099)
  Probe inputs: a multi-digit decimal (e.g. `123`), `4'b1010`,
  `8'hDEAD`, a multi-digit nonzero, `3'o17` — confirm `rest` emits
  `<invalid_sequence_access>`. Recommended fix: lift
  `( kw_sv_rule_c82a06f6 | digit )` into a named rule and adopt the
  bare-`rest:$2` named-lead idiom (parent confirms whether the
  `sv_rule` group-separator token should appear in the digit list or
  be dropped).
  **DONE — RESOLVED — FIXED-DEFENSIVE in systemverilog 1.0.116 /
  schema 2, `PGEN-POST-SV-AUDIT-0005` (leaf POST-SV-AUDIT.2.4a,
  2026-05-17).** The inline alternation in each rule was lifted into a
  new **un-annotated** named tail rule
  (`unsigned_number_tail := kw_sv_rule_c82a06f6 | decimal_digit`, …) so
  `( <rule>_tail )*` lets bare `$2` bind cleanly; the
  `{first: $1, rest: $2}` annotation text is **unchanged** (identical
  transformation proven 6× this session — correct by construction).
  **Honest disposition:** the parent empirically established the
  corruption is **structurally present but NOT consumer-reproducible**
  — the SV `systemverilog_file` root **rejects every numeric-bearing
  top-level construct** (`parameter`/`localparam`/`assign`/`$display`/
  packed ranges/module-param headers) in **all** profiles
  (`default`/`sv_2017`/`sv_2023`); only minimal constructs parse, so a
  multi-digit number is unreachable via valid `source_text` (a
  pre-existing SV-grammar-root coverage limitation, separate from this
  defect and explicitly out of POST-SV-AUDIT scope). This is therefore
  a **defensive structural correction**, **NOT** a
  `PGEN_RELEASED_PARSER_BUG_LEDGER` row (we do not claim a released
  defect that no valid input can trigger — this is the honest
  disposition). `kw_sv_rule_c82a06f6 := trivia /sv_rule\b/` is a
  degenerate LRM-extraction artifact (separate grammar-quality matter,
  out of scope — observation only). See the RESOLVED note in the
  systemverilog.ebnf section above.

### Structured-per-iteration Cat-A (objective bug, medium, needs parent judgement on the exact extraction shape)

`X … ( SEP X … )*` where each iteration carries a multi-field record
(not a single element). Raw `{first:{…},rest:$N}` exposes
`[[SEP, field, field, …],…]`. Objective usability bug, but the fix is
not a one-liner `[$N, $M::2*]` — it needs the repeated unit factored
into a NAMED rule that emits the record, then an extraction-spread.
Parent to confirm the desired record shape per rule.

- rtl_frontend.ebnf: (none — all rtl_frontend list rules are
  single-element; covered above).
- systemverilog.ebnf: `list_of_interface_identifiers` (2435-2436),
  `list_of_port_identifiers` (2490-2491),
  `list_of_tf_variable_identifiers` (2499-2500),
  `list_of_variable_identifiers` (2514-2515),
  `list_of_variable_port_identifiers` (2517-2518),
  `let_list_of_arguments` named_only (2357-2358),
  `parameter_port_list` type_only (3327-3328),
  `assignment_pattern` named (3385-3386 and 3401-3402),
  `property_list_of_arguments` named_only (3800-3801),
  `sequence_list_of_arguments` named_only (4197-4198).
  In-grammar model: the sibling single-element lists
  (`list_of_genvar_identifiers` l.2432, `list_of_specparam_assignments`
  l.2496, `list_of_variable_decl_assignments` l.2511, the `ordered`
  assignment_pattern branch, `parameter_port_list` declarations
  branch) already use the correct `[$1, $2::2*]` — that is the
  reference idiom.

## Cat-C / benign / already-correct (for POST-SV-AUDIT.3 to confirm)

- Cat-B-resolved (named op-rule + bare `rest`/`lhs`): rtl_const_expr
  all 10 binop_chain rules (19-38); rtl_frontend 10 binop_chain rules
  (200-227); vhdl `expression`/`relation`/`simple_expression`/`term`/
  `factor` (348-366); systemverilog `constant_expression` (285),
  `expression_base` operand_chain (297),
  `module_path_expression` chain (2803). Confirmed correct — do NOT
  re-fix. The closed systemic binop class (RTL-CE-0001 / SVPP-0001 /
  RTL-FE-0001 / VHDL-0001) is exhaustively re-verified here: every
  binop level in every product grammar uses a named op-rule.
- Cat-C bare `X X*` repetition (`{first,rest}` benign, `rest` already
  a clean array): systemverilog `case_generate_construct` (694),
  `case_statement` (734), `combinational_body` (1029),
  `data_type` struct_union (1501), `randcase_statement` (3926),
  `randsequence_statement_sv_2017`/`_sv_2023` (3936, 3940),
  `rs_case` (3974), `rs_production_list_sv_2017`/`_sv_2023`
  productions+rand_join (4043, 4045, 4049, 4051),
  `sequential_body` (4218),
  `udp_declaration_sv_2017`/`_sv_2023` nonansi (4790, 4802).
- Not-an-iteration (out of the A/B/C taxonomy, benign): systemverilog
  `delay_sv_2017`/`_sv_2023` triple_optional (1589, 1597) — `$N`
  references an OPTIONAL `( … )?` group (fixed arity ≤ 3), not a
  `*`/`+` iteration; `list_of_arguments_mixed_head` chain (2410) —
  `$3` is a single recursive-rule node, not an iteration envelope.
  Style-only; recorded, not churned.
- regex.ebnf: zero in-scope occurrences (different annotation idiom).

## Reconciliation vs the pre-scope counts

`{first/lhs..rest:$N}` audit-surface counts — the pre-scope measured
the `rest:$N` carrier lines (the iteration-bearing position):

| Grammar | Pre-scope | Enumerated (`rest:$N` lines) | Match? |
| --- | --- | --- | --- |
| rtl_const_expr | 10 | 10 | yes |
| rtl_frontend | 28 | 27 real annotations + 1 comment line (`rtl_frontend.ebnf:214` documents the RTL-FE-0001 fix and contains `\`rest:$2\`` in backticks) = 28 grep hits | yes (the 28th is a comment, not an occurrence) |
| systemverilog_preprocessor | 1 | 1 | yes |
| systemverilog | 38 | 38 | yes |
| vhdl | 22 | 22 | yes |
| regex | 0 | 0 | yes |

Raw `{first:$1, rest:$2}` candidates:

| Grammar | Pre-scope | Enumerated | Match? |
| --- | --- | --- | --- |
| rtl_frontend | 3 | 3 (l.80 `parameter_declaration_sequence`, 99 `port_list`, 262 `scoped_identifier`) | yes |
| systemverilog | 7 | 7 (l.345, 544, 2062, 3064, 3099 = the 5 inline-alt number rules; 4043, 4049 = the two Cat-C `productions` `{first:$1,rest:$2}` branches) | yes |
| vhdl | 7 | 7 (l.43, 45, 63, 74, 277, 324, 384) | yes |

Note: the strict-literal `{first: $1, rest: $2}` SV subset is exactly
the 5 HIGH-priority inline-alt number rules plus the two benign Cat-C
`rs_production_list` `productions` branches — NOT the structured
`{first:{name:$1,dims:$2},rest:$3}` list rules (those use nested
objects and do not match the strict literal). All 38 SV `rest:$N`
lines are enumerated and classified above regardless of which raw
subset a literal grep matches.

No unexplained mismatch. The only nuances are: (a) the rtl_frontend
"28" pre-scope hit count includes one documentation comment line
(`rtl_frontend.ebnf:214`, the RTL-FE-0001 fix note containing
`` `rest:$2` `` in backticks) — there are 27 real annotation
occurrences; (b) the SV raw-literal "7" is a strict-syntax slice of
the full 38 `rest:$N` lines (all 38 are classified above).
