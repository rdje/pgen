# docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `rtl_frontend` parser family.

This is the document downstream projects (primarily RTLSyn) embedding the PGEN rtl_frontend parser should read first.

## Contract Identity
- Contract version:
  - `1.0.1`
- Parser release version:
  - `1.0.1`
- Embedding API contract baseline:
  - tracked under `rust/docs/EMBEDDING_API_CONTRACT.md`
- rtl_frontend AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-15`
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

1. **Parser release version** (`1.0.1`). Tracks the parser library's release identity. Bumped on every functional change.
2. **AST-dump schema version** (`1`). Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 1.0.0 | 1.0.1 | **RTL-FE-Slice-1..7** — initial 156-annotation baseline. Dispatch wrappers (design_item/package_item/module_item/generate_item), keyword leaves, expression dispatch + 10-rule binop_chain hierarchy, declarations + module structure, parameter/port rules, module instantiation/ports/statements/signals/datatypes mass batch. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/rtl_frontend.ebnf`) un-annotated except for `rtl_frontend_file -> {type, items}` root. AST dump is the recursive-envelope shape across all rules. |

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

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `rtl_frontend` is an `In Progress` family in the live tracker. The current grammar covers the synthesizable RTL subset; the full IEEE 1800 SystemVerilog surface is **out of scope** — see the `systemverilog` family for that.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
