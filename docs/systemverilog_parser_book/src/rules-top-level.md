# Top-Level Rules

This chapter describes the entry points of the SystemVerilog grammar and the AST shape they produce.

> **Status:** SV-Slice-1 (parser release `1.0.1`) typed `systemverilog_file` and `systemverilog_parseable_file`. SV-Slice-2 (parser release `1.0.2`) flattened `source_text` via `[$1**]`. SV-Slice-3 (parser release `1.0.3`) typed `source_text_item` per-branch. SV-Slice-4 (parser release `1.0.4`) typed `description` per-branch with `kind:` discriminator. SV-Slice-5 (parser release `1.0.5`) made `compiler_directive` transparent passthrough. SV-Slice-6 batch (parser release `1.0.6`) typed `attribute_instance` (`{first, rest}` shape) and `module_declaration_sv_2017` / `module_declaration_sv_2023` per-branch. SV-Slice-7 batch (parser release `1.0.7`) typed `module_keyword`, `lifetime`, `module_ansi_header`, `module_nonansi_header` — the module-header sub-tree. SV-Slice-8 batch (parser release `1.0.8`) typed the 4 identifier-leaf rules (`simple_identifier`, `escaped_identifier`, `non_keyword_identifier`, `simple_identifier_no_scope`) so clean identifier strings propagate through every typed parent rule's name field. SV-Slice-9 batch (parser release `1.0.9`) typed interface declarations (full mirror of module pattern). SV-Slice-10 batch (parser release `1.0.10`) typed class declarations, `package_declaration`, and program declarations (5 per-branch kinds each). SV-Slice-11 batch (parser release `1.0.11`) typed `program_ansi_header` and `program_nonansi_header`. SV-Slice-12 batch (parser release `1.0.12`) typed the UDP declaration family. SV-Slice-13 batch (parser release `1.0.13`) typed `bind_directive` (2 kinds), `bind_instantiation` (4 kinds), and `package_item` (4 kinds). SV-Slice-14 batch (parser release `1.0.14`) typed the rest of the bind sub-tree, `interface_class_declaration`, and `config_declaration`. SV-Slice-15 batch (parser release `1.0.15`) typed the port-list family + `anonymous_program` and `package_export_declaration`. SV-Slice-16 batch (parser release `1.0.16`) typed `port`, `port_direction`, `package_import_declaration`, `package_import_item`. SV-Slice-17 batch (parser release `1.0.17`) typed the UDP body sub-tree. SV-Slice-18 batch (parser release `1.0.18`) typed UDP truth-table entries. SV-Slice-19 batch (parser release `1.0.19`) typed the module-items dispatch tree (`module_item`, `module_or_generate_item`, `module_or_generate_item_declaration`, `non_port_module_item`, `continuous_assign`) — every `header.items` / `body.items` field on every typed module/interface/program declaration now exposes kind-discriminated dispatch. SV-Slice-20 batch (parser release `1.0.20`) mirrored that batch onto interface/program (`interface_item`, `interface_or_generate_item`, `non_port_interface_item`, `program_item`, `non_port_program_item`) — interface and program walks now match the module walk's typed-dispatch level (5 rules / 19 annotations). SV-Slice-21 batch (parser release `1.0.21`) typed `module_common_item` (both profiles) and `package_or_generate_item_declaration` (both profiles) — biggest batch yet at 4 rules / 55 annotations; closes the cascading walk path that SV-Slice-19/20 set up so every reachable common-item and package-or-generate-item-declaration discriminates its matched sub-construct. SV-Slice-22 batch (parser release `1.0.22`) typed the generate sub-tree: `generate_region` (`{items}`), `generate_item` (3 kinds), and `generate_block` (anonymous / labeled / generate_item passthrough) — closes the generate-construct walk path; 3 rules / 7 annotations. SV-Slice-23 batch (parser release `1.0.23`) typed the generate-construct internals: `loop_generate_construct` (`{init, condition, step, block}`), `conditional_generate_construct` (2 kinds), `if_generate_construct` (`{condition, then_block, else_clause}`), NEW helper rule `if_generate_else_clause` (2 kinds — workaround for task #38), `case_generate_construct` (`{expr, items}`), `case_generate_item` (2 kinds); 6 rules / 9 annotations + 1 new helper rule. SV-Slice-24 batch (parser release `1.0.24`) typed the assertion + genvar dispatch: `assertion_item` (2 kinds), `assertion_item_declaration` (3 kinds), `concurrent_assertion_item` (2 kinds), `genvar_initialization` (`{genvar_keyword, name, value}`), `genvar_iteration` (3 kinds), `assignment_operator` (13 kinds — bare `{kind}`), `inc_or_dec_operator` (2 kinds — bare `{kind}`); 7 rules / 26 annotations. SV-Slice-25 batch (parser release `1.0.25`) typed data/function/task declarations + bodies: `data_declaration_sv_2017/2023` (4 kinds each), `function_declaration_sv_2017/2023` (`{lifetime, body}` / `{dynamic_override, lifetime, body}`), `function_body_declaration` (`{return_type, name, items, statements, end_label}`), `task_declaration_sv_2017/2023` (parallel to function), `task_body_declaration` (no return_type); 8 rules / 14 annotations. SV-Slice-26 batch (parser release `1.0.26`) typed `net_declaration_sv_2017/2023` (3 kinds each: wire / alias / interconnect) using NEW helper rules `net_strength` (2 kinds: drive / charge) and `net_vector_scalar` (2 kinds: vectored / scalared, bare `{kind}`) — extracted from inline parens-Or to dodge task #38; 4 rules / 10 annotations + 2 helper rules. SV-Slice-27 batch (parser release `1.0.27`) typed the class body sub-tree: `class_item_sv_2017/2023` (8/9 kinds), `class_item_qualifier` (3 kinds bare), `class_constraint` (2 kinds), `class_property` (2 kinds: decl / const), `class_method` (6 kinds: task / function / pure_virtual / extern / constructor / extern_constructor); 6 rules / 30 annotations.

## Entry points by profile

| Profile | Entry rule | Description |
|---|---|---|
| `sv_2017` | `systemverilog_file` (per LRM Annex A.1.1) | IEEE 1800-2017 source file. |
| `sv_2023` | `systemverilog_file` (per LRM Annex A.1.1, with 2023 deltas) | IEEE 1800-2023 source file. Same entry rule symbol; the 2023 grammar differs in interior rules where the LRM was extended. |

Both profiles share `grammars/systemverilog.ebnf` as the single source. The profile selection determines which top-level dispatcher rule is used at parse time.

## `systemverilog_file` (typed since SV-Slice-1)

Per `grammars/systemverilog.ebnf` line 184:

```ebnf
systemverilog_file := trivia source_text trivia
                   -> {type: "systemverilog_file", source_text: $2}
```

The annotation produces a typed JSON object at the root of every `sv_2017` / `sv_2023` parse. For an input like `"module m; endmodule\n"`:

```json
{
  "type": "systemverilog_file",
  "source_text": [/* source_text envelope */]
}
```

The `source_text` field is a flat array of `source_text_item` shapes (since SV-Slice-2 — see [`source_text` (flat array since SV-Slice-2)](#source_text-flat-array-since-sv-slice-2) below). Consumers walking the SV AST should dispatch on `obj["type"] == "systemverilog_file"` at the root level.

## `source_text` (flat array since SV-Slice-2)

Per `grammars/systemverilog.ebnf` line 2273:

```ebnf
source_text := source_text_item*
            -> [$1**]
```

The `[$1**]` flatten-spread idiom produces a clean flat array of `source_text_item` shapes. Pre-SV-Slice-2 this was the raw Quantified envelope of the iteration; consumers walking `obj["source_text"]` had to descend through a Quantified wrap. Post-fix the array is consumer-ready — iterate directly:

```rust
for item in obj["source_text"].as_array().unwrap() {
    walk_source_text_item(item);
}
```

For `module m; endmodule\n`, `source_text` has length 1 (the single `module_declaration` source-text item). For a multi-construct file (multiple modules + interfaces + packages), it carries one item per top-level construct in source order.

The inner `source_text_item` shapes are still **raw envelope** (Or of `description | local_parameter_declaration semi | parameter_declaration semi | package_import_declaration | bind_directive | ...`). Per-branch typing of source_text_item is a follow-up slice (will assign each branch a `kind:` discriminator).

See `rust/test_data/ast_shape_contract/systemverilog_v1.json` for the calibrated regression-lock sample.

## `description` (un-annotated)

`description` is the per-construct top-level alternative — module / interface / class / package / etc. Per LRM A.1.2:

```ebnf
description = module_declaration
            | udp_declaration
            | interface_declaration
            | program_declaration
            | package_declaration
            | (attribute_instance* package_item)
            | (attribute_instance* bind_directive)
            | config_declaration
```

Un-annotated `description` produces the matched-branch shape directly (no extra wrapping). When a slice annotates the per-branch dispatch, this rule will likely become:

```ebnf
description -> {type: "description", kind: "<branch>", body: $1}
```

(or similar) — actual annotation lands per-slice with the corresponding manifest update.

## After the first slice

Once the first annotation slice lands, this chapter will document:

- The `-> ...` annotation that landed on each top-level rule.
- A worked example (input → JSON tree) for `module m; endmodule\n`.
- The cumulative manifest entries.

For now, consult the [Walking the AST](walking-the-ast.md) walker pattern and use the recursive-envelope walk path for top-level rules.

## How to follow per-slice changes

Each annotation slice gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`. A row in the [Changelog Index](changelog-index.md) ties the two together with a one-paragraph summary.
