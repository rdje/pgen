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
> `1`** tracked in [Schema Versioning](#schema-versioning), the grammar
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
| `"genvar"` | `{first, rest}` — comma-separated genvar identifier list | `genvar_declaration` (line 120) |
| `"module_instantiation"` | `{module_name, parameters, first, rest}` — `parameters` is `[]` when no `#(…)` override; `first` + `rest` are the `instance_item` instances | `module_instantiation` (line 123) |
| `"net"` | `{data_type, packed_range, first, rest}` — `first` + `rest` are the `net_item` declarators | `net_declaration` (line 144) |
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
| `"genvar"` | `{first, rest}` | `genvar_declaration` (line 120) |
| `"module_instantiation"` | `{module_name, parameters, first, rest}` | `module_instantiation` (line 123) |
| `"net"` | `{data_type, packed_range, first, rest}` | `net_declaration` (line 144) |
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
enumerates the full typed surface of contract `1.0.1`
(**156 annotations across 74 distinct rules**, schema version `1`); this
contract section is curated, the inventory artifact is the authoritative
machine-checkable enumeration, and this integration contract wins over the
per-family book if they ever disagree.

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `rtl_frontend` is an `In Progress` family in the live tracker. The current grammar covers the synthesizable RTL subset; the full IEEE 1800 SystemVerilog surface is **out of scope** — see the `systemverilog` family for that.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
