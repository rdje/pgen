# Top-Level Rules

This chapter describes the entry points of the SystemVerilog grammar and the AST shape they produce.

> **Status:** Most top-level rules are currently un-annotated and produce the recursive-envelope shape. The first slice of the campaign will land here. Until then, this chapter documents the envelope shape for orientation.

## Entry points by profile

| Profile | Entry rule | Description |
|---|---|---|
| `sv_2017` | `systemverilog_file` (per LRM Annex A.1.1) | IEEE 1800-2017 source file. |
| `sv_2023` | `systemverilog_file` (per LRM Annex A.1.1, with 2023 deltas) | IEEE 1800-2023 source file. Same entry rule symbol; the 2023 grammar differs in interior rules where the LRM was extended. |

Both profiles share `grammars/systemverilog.ebnf` as the single source. The profile selection determines which top-level dispatcher rule is used at parse time.

## `systemverilog_file` (un-annotated)

Per the LRM and `grammars/systemverilog.ebnf`, the top-level production is something like:

```ebnf
systemverilog_file = (timeunits_declaration?  description*)
```

Because there is no `-> ...` annotation, the rule produces the recursive-envelope shape. For an input like `"module m; endmodule\n"`, the `root` field of `AstDumpPayload` is a JSON array reflecting the matched grammar shape:

```text
[<timeunits_declaration_or_empty>, <description_iterations>]
```

The exact JSON shape until the first annotation slice lands depends on the un-annotated emitter behavior; see `rust/test_data/ast_shape_contract/systemverilog_v1.json` for the calibration sample (`current_content_kind` field).

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
