# Glossary

Terms used throughout this book. Where a term has a normative definition, the integration contract `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` is authoritative; this glossary paraphrases it for quick lookup.

## AST envelope

The typed rtl_frontend AST. It is obtained by **parsing the `dump_json` string** of the `AstDumpPayload` (the real struct has exactly `dump_json`/`truncated`/`full_bytes`/`emitted_bytes` — there is **no** `root` field). After confirming `truncated == false`, `serde_json::from_str(&payload.dump_json)` yields the `rtl_frontend_file` root object this book's per-rule chapters describe. See [AST Envelope Structure](ast-envelope.md).

## AST shape contract manifest

The file `rust/test_data/ast_shape_contract/rtl_frontend_v1.json` (`version: 1`, `grammar: "rtl_frontend"`). It records the per-rule expected JSON shape for each sample in the rtl_frontend test corpus (currently the `empty_module` sample) and embeds the declared-annotation inventory — **156 annotation entries**. Drift in the AST dump fails the `rtl_frontend_ast_shape_contract_holds_against_running_generated_parser` regression-lock test under `cargo test --lib --features generated_parsers rtl_frontend_ast_shape_contract`, surfacing the change. Its `declared_annotation_inventory.annotations` list is content-identical to the live inventory `generated/rtl_frontend_return_annotations.json` (the live inventory additionally carries a `raw_text` field per entry; the `rule` / `branch_index` / `annotation_type` / `normalized_text` tuples match exactly). (This is the rtl_frontend manifest; the SystemVerilog and VHDL parsers have their own separate `systemverilog_v1.json` / `vhdl_v1.json`.)

## binop_chain

The consumer-facing left-fold contract for rtl_frontend's expression-precedence hierarchy. rtl_frontend has a **ten-level** binary-operator cascade — `logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr`. Every level emits the same `{type: "binop_chain", level, lhs, rest}` shape, so a single consumer fold handles the whole expression tree. `lhs` is the leading operand (itself a `binop_chain` of the next-tighter level); `rest` is the recursive-envelope iteration array of `(operator, operand)` pairs folded left-associatively. Unlike VHDL's `simple_expression`, **no rtl_frontend level carries a `sign` field** — prefix `+` / `-` / `!` / `~` live in the `unary_expr` rule below the cascade, and `conditional_expr` (ternary) / `unary_expr` are passthrough when their distinguishing syntax is absent. See [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract) for the level/field/operator table and [Walking the AST](walking-the-ast.md) for the fold code.

## Declared-annotation inventory

The machine-checkable enumeration of every typed-shape annotation the rtl_frontend grammar emits: `generated/rtl_frontend_return_annotations.json` (`version: 1`, `grammar: "rtl_frontend"`, `annotation_count: 156`). It is the live source of truth for the typed surface and is mirrored content-for-content by the embedded inventory in `rust/test_data/ast_shape_contract/rtl_frontend_v1.json` (156 entries). Of the 156 entries, 154 are `return_object` annotations and exactly **two** are `return_scalar` positional passthroughs (`conditional_expr` and `unary_expr`). If this book's prose disagrees with the inventory, the inventory wins; if the inventory disagrees with the integration contract, the contract wins.

## design_item dispatch

The primary top-level dispatcher of the rtl_frontend AST. `design_item` is a 4-branch `kind`-tagged shape — `"typedef"`, `"package"`, `"module"`, `"semi"` — each carrying a `body` (the `"semi"` branch is bodyless, a lone `;` separator). Every parse roots at `{type: "rtl_frontend_file", items: [...]}`; each element of `items` is a `design_item` object. This is the only rule that carries a `type` discriminator at the dispatch level; every other dispatcher uses `kind`. See [AST Envelope Structure](ast-envelope.md) and [Top-Level Rules](rules-top-level.md).

## {first, rest} list convention

The uniform carrier for rtl_frontend separated lists (`port_list`, `parameter_declaration_sequence`, `genvar_declaration`, `net_declaration`, `scoped_identifier`, `event_control_list`, `module_instantiation`, `concatenation_expr`, `parameter_override_list`, `port_connection_list`, `struct_union_field`, `enum_type`, …). Each list emits `{first: <head-element>, rest: <iteration-of-the-(separator element)*-tail>}`. `rest` is a recursive-envelope array; each entry is the envelope of one `(separator element)` iteration. Unlike the SystemVerilog grammar — whose lists were flattened to clean `[$N, $M::2*]` arrays in its slice-58 audit — the rtl_frontend grammar uses `{first, rest}` uniformly across RTL-FE-Slice-1..7, so consumers should expect to descend through the separator wrap when iterating a list. A future flattening slice, if it lands, gets its own [Changelog Index](changelog-index.md) row. See [The Json Carrier](json-carrier.md) and [Walking the AST](walking-the-ast.md) for the iteration helper.

## parseability_probe

The CLI wrapper around `pgen::embedding_api` used for terminal-side verification, AST inspection, and bug-report reproducers. Sub-commands include `--parse`, `--parse-dump-ast`, and `--parse-dump-ast-pretty`. For rtl_frontend the parser is on-demand-only, so the probe must be built with the generated backend before use (see [Build Recipe](build-recipe.md)). The full flag set, exit codes, and registered grammars are in the [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md).

## Parser release version

The parser library's release identity, currently `1.0.1`. Bumped on every functional change to the parser, including bug fixes, performance work, and grammar changes. It moves independently of the schema version: a release can carry the same schema version as the previous one (no shape change) or a bumped one (shape changed). Recorded in `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". See [Schema Versioning](schema-versioning.md).

## Profile

A named configuration of the grammar that selects which top-level entry rule to start parsing from. rtl_frontend has exactly **one** profile: `default`, whose entry rule is `rtl_frontend_file` (the synthesizable RTL subset source file). There is **no** per-grammar convenience function for rtl_frontend (no `parse_rtl_frontend`); the stable host surface is the generic-by-grammar entry points — `parse_grammar_profile`, `parse_grammar_profile_result`, `parse_grammar_profile_ast_dump` with `GrammarFamily::RtlFrontend` + `GrammarProfile::Default`, plus the `parse_grammar_profile_named` string overload with `"rtl_frontend"` / `"default"`. See [Public API Surface](public-api.md).

## Recursive envelope

The default JSON shape produced by un-annotated rules — a recursive composition of arrays (for sequences, quantified iterations, and the `rest` tail of a `{first, rest}` list), strings (for terminal and regex leaves), and matched-branch passthroughs (for alternations). Un-matched optionals are the empty array `[]`, never `null`. In rtl_frontend the recursive envelope is what you reach when you descend below the typed surface: `identifier` tokens, `named_data_type`, the passthrough forms of `conditional_expr` / `unary_expr`, and the few utility rules with no per-rule annotation. See [AST Envelope Structure](ast-envelope.md) and [The Json Carrier](json-carrier.md).

## Return annotation

A `-> ...` clause appended to a grammar rule definition in `grammars/rtl_frontend.ebnf` that overrides the default recursive-envelope shape with a typed JSON value. Example: `rtl_frontend_file := trivia design_item* trivia -> {type: "rtl_frontend_file", items: $2}`. The annotation language (`$N` positional references, `{field: value}` object literals, `[...]` array literals, string literals) is specified in `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. rtl_frontend uses two annotation kinds: `return_object` (an object literal) and `return_scalar` (a positional passthrough such as `-> $1` on the `conditional_expr` / `unary_expr` non-distinguishing branches).

## Schema version

Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to (a new annotation on a previously-unannotated rule, a restructured annotation, a user-visible grammar-shape change). Pure performance work and internal codegen reorganization do **not** bump it. The AST-dump schema version is currently `1` — note it is **not** a field of `AstDumpPayload` (that struct has only `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`); consumers **pin** the schema version they built against from `docs/contracts/PGEN_RTL_FRONTEND_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" and re-check the contract's "Schema Versioning" table when bumping PGEN. That table additionally uses `1.0.0` / `0.1.0` milestone labels for the typing-campaign timeline. See [Schema Versioning](schema-versioning.md).

## Typed shape

The JSON value produced by an annotated rule. In rtl_frontend it takes three sub-forms: a root object carrying `type` (only `rtl_frontend_file`); a `kind`-tagged dispatch object (`{kind, body?}` or per-branch named fields); and a named-field object for single-sequence rules. The `binop_chain` expression rules combine the `type` and named-field forms (`{type: "binop_chain", level, lhs, rest}`), and `conditional_expr`'s ternary form carries a `type`. Consumers dispatch on `obj["type"]` at the root and `obj["kind"]` for variants. See [The Json Carrier](json-carrier.md) and [Top-Level Rules](rules-top-level.md).

## Un-annotated leaf (rule of thumb)

A field bound to an un-annotated rule — most importantly every identifier-valued field (`name`, `module_name`, `genvar`, …) — surfaces as that rule's **recursive envelope**, not a bare JSON string. The identifier text is nested inside the envelope, reached by walking to the terminal. Do not assume `obj["name"]` is a string: the rtl_frontend `identifier` rule is a large alternation, so its envelope carries one `[]` slot per unmatched alternative and the matched text near the end. The robust consumer rule is to walk to the single non-`[]` element and read its trailing string rather than indexing a fixed depth/offset. See [The Json Carrier](json-carrier.md) and the worked walkthrough in [Empty Module](examples-empty-module.md).

## RTL-FE-Slice-1..7

The seven typing slices that landed the entire `grammars/rtl_frontend.ebnf` typed surface — **156 return annotations on 74 distinct rules**, parser release `1.0.1`, schema version `1`, all landed on 2026-05-14. Unlike the VHDL grammar, which was typed in a single comprehensive batch (`VHDL-Slice-1`), and unlike the SystemVerilog and regex parsers, whose return annotations were added rule-by-rule over a long per-slice campaign (each slice bumping the schema version), rtl_frontend was typed across a small number of grouped slices: dispatch wrappers (slice 1), keyword/operator leaves (slice 2), expression dispatch + procedural (slice 3), the ten-rule `binop_chain` hierarchy (slice 4), declarations + module structure (slice 5), parameter/port rules (slice 6), and the module-instantiation / ports / statements / signals / datatypes mass batch (slice 7). This is why the rtl_frontend [Changelog Index](changelog-index.md) and [Schema Versioning](schema-versioning.md) timeline are short by design. Subsequent shape-affecting slices, if any, each get their own contract row and changelog entry.
