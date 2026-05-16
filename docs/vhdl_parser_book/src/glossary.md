# Glossary

Terms used throughout this book. Where a term has a normative definition, the integration contract `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` is authoritative; this glossary paraphrases it for quick lookup.

## AST envelope

The typed VHDL AST. It is obtained by **parsing the `dump_json` string** of the `AstDumpPayload` (the real struct has exactly `dump_json`/`truncated`/`full_bytes`/`emitted_bytes` — there is **no** `root` field). After confirming `truncated == false`, `serde_json::from_str(&payload.dump_json)` yields the `vhdl_file` root object this book's per-rule chapters describe. See [AST Envelope Structure](ast-envelope.md).

## AST shape contract manifest

The file `rust/test_data/ast_shape_contract/vhdl_v1.json`. It records the per-rule expected JSON shape for each sample in the VHDL test corpus and embeds the declared-annotation inventory (`version: 1`, 249 entries). Drift in the AST dump fails the `vhdl_ast_shape_contract_holds_against_running_generated_parser` regression-lock test under `cargo test`, surfacing the change. Its content is byte-identical to the live inventory `generated/vhdl_return_annotations.json`. (This is the VHDL manifest; the SystemVerilog parser has its own separate `systemverilog_v1.json`.)

## binop_chain

The consumer-facing left-fold contract for VHDL's five-level expression-precedence hierarchy. Every level — `expression` (logical), `relation` (relational), `simple_expression` (additive), `term` (multiplicative), `factor` (power) — emits the same `{type: "binop_chain", level, lhs, rest}` shape, so a single consumer fold handles the whole expression tree. `lhs` is the leading operand; `rest` is the iteration array of `(operator, operand)` pairs folded left-associatively; `simple_expression` adds a leading `sign` field for the optional unary `+`/`-`. See [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract) for the level/field/operator table and [Walking the AST](walking-the-ast.md) for the fold code.

## Declared-annotation inventory

The machine-checkable enumeration of every typed-shape annotation the VHDL grammar emits: `generated/vhdl_return_annotations.json` (`version: 1`, `grammar: "vhdl"`, `annotation_count: 249`). It is the live source of truth for the typed surface and is mirrored byte-for-byte by the embedded inventory in `rust/test_data/ast_shape_contract/vhdl_v1.json`. If this book's prose disagrees with the inventory, the inventory wins; if the inventory disagrees with the integration contract, the contract wins.

## design_unit dispatch

The primary top-level dispatcher of the VHDL AST. `design_unit` is a 10-branch `kind`-tagged shape — `"library"`, `"use"`, `"context_reference"`, `"entity"`, `"architecture"`, `"package"`, `"package_body"`, `"configuration"`, `"context"`, `"semi"` — each carrying a `body` (the `"semi"` branch is bodyless). Every parse roots at `{type: "vhdl_file", design_units: [...]}`; each element of `design_units` is a `design_unit` object. See [AST Envelope Structure](ast-envelope.md) and [Top-Level Rules](rules-top-level.md).

## {first, rest} list convention

The uniform carrier for VHDL separated lists (`identifier_list`, `selected_name`, `association_list`, `library_clause`, `use_clause`, `parameter_list`, `choices`, `enumeration_type_definition`, …). Each list emits `{first: <head-element>, rest: <iteration-of-the-(separator element)*-tail>}`. `rest` is a recursive-envelope array; each entry is the envelope of one `(separator element)` iteration. Unlike the SystemVerilog grammar — whose lists were flattened to clean `[$N, $M::2*]` arrays in its slice-58 audit — the VHDL grammar uses `{first, rest}` uniformly across VHDL-Slice-1. A future flattening slice, if it lands, gets its own [Changelog Index](changelog-index.md) row. See [Walking the AST](walking-the-ast.md) for the iteration helper.

## parseability_probe

The CLI wrapper around `pgen::embedding_api` used for terminal-side verification, AST inspection, and bug-report reproducers. Sub-commands include `--parse`, `--parse-dump-ast`, and `--parse-dump-ast-pretty`. For VHDL the parser is on-demand-only, so the probe must be built with the generated backend before use (see [Build Recipe](build-recipe.md)). The full flag set, exit codes, and registered grammars are in the [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md).

## Parser release version

The parser library's release identity, currently `1.0.1`. Bumped on every functional change to the parser, including bug fixes, performance work, and grammar changes. It moves independently of the schema version: a release can carry the same schema version as the previous one (no shape change) or a bumped one (shape changed). Recorded in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". See [Schema Versioning](schema-versioning.md).

## Profile

A named configuration of the grammar that selects which top-level entry rule to start parsing from. VHDL has exactly **one** stable profile: `vhdl_1076_2019` (IEEE 1076-2019), whose entry rule is `vhdl_file`. The host entry points for it are `parse_vhdl_1076_2019` and `parse_vhdl_1076_2019_ast_dump`. Recognized profile-name aliases: `"vhdl_1076_2019"`, `"1076-2019"`, `"ieee1076-2019"`, `"ieee_1076_2019"`. See [Public API Surface](public-api.md).

## Recursive envelope

The default JSON shape produced by un-annotated rules — a recursive composition of arrays (for sequences, quantified iterations, and the `rest` tail of a `{first, rest}` list), strings (for terminal and regex leaves), and matched-branch passthroughs (for alternations). Un-matched optionals are the empty array `[]`, never `null`. In VHDL the recursive envelope is what you reach when you descend below the typed surface: identifier tokens, physical / bit-string / string / character literals, and the few utility rules with no per-rule annotation. See [AST Envelope Structure](ast-envelope.md) and [The Json Carrier](json-carrier.md).

## Return annotation

A `-> ...` clause appended to a grammar rule definition in `grammars/vhdl.ebnf` that overrides the default recursive-envelope shape with a typed JSON value. Example: `vhdl_file := design_unit* -> {type: "vhdl_file", design_units: $1}`. The annotation language (`$N` positional references, `{field: value}` object literals, `[...]` array literals, string literals) is specified in `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Schema version

Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to (a new annotation on a previously-unannotated rule, a restructured annotation, a user-visible grammar-shape change). Pure performance work and internal codegen reorganization do **not** bump it. The AST-dump schema version is currently `1` — note it is **not** a field of `AstDumpPayload` (that struct has only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`); consumers **pin** the schema version they built against from `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" and re-check the contract's "Schema Versioning" table when bumping PGEN. That table additionally uses `1.0.0` / `0.1.0` milestone labels for the typing-campaign timeline. See [Schema Versioning](schema-versioning.md).

## Typed shape

The JSON value produced by an annotated rule. In VHDL it takes three sub-forms: a root object carrying `type` (only `vhdl_file`); a `kind`-tagged dispatch object (`{kind, body}` or per-branch named fields); and a named-field object for single-sequence rules. The `binop_chain` expression rules combine the `type` and named-field forms. Consumers dispatch on `obj["type"]` at the root and `obj["kind"]` for variants. See [The Json Carrier](json-carrier.md) and [Top-Level Rules](rules-top-level.md).

## VHDL-Slice-1

The single comprehensive typing batch that landed the entire `grammars/vhdl.ebnf` typed surface at once — **110 distinct rules / 249 return annotations**, parser release `1.0.1`, schema version `1`. Unlike the SystemVerilog and regex parsers, whose return annotations were added rule-by-rule over many slices (each slice bumping the schema version), the VHDL grammar was annotated line-by-line in one pass. This is why the VHDL [Changelog Index](changelog-index.md) and [Schema Versioning](schema-versioning.md) timeline are short by design. Subsequent shape-affecting slices, if any, each get their own contract row and changelog entry.
