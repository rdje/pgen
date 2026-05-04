# Top-Level Rules

This chapter describes the entry points of the SystemVerilog grammar and the AST shape they produce.

> **Status:** SV-Slice-1 (parser release `1.0.1`) typed `systemverilog_file` and `systemverilog_parseable_file`. The rest of the top-level rules are un-annotated and produce the recursive-envelope shape; subsequent slices will type them.

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

The `source_text` field carries the raw envelope of the `source_text` rule until that rule is annotated in a subsequent slice. Consumers walking the SV AST should dispatch on `obj["type"] == "systemverilog_file"` at the root level.

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
