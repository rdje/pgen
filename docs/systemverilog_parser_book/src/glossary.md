# Glossary

Terms used throughout this book.

## AST envelope
The JSON tree that `parse_grammar_profile_ast_dump_named` returns. See [AST Envelope Structure](ast-envelope.md).

## AST shape contract manifest
The file `rust/test_data/ast_shape_contract/systemverilog_v1.json`. Records per-rule expected JSON shape for each entry in the test corpus. Drift in the AST dump fails a `cargo test` regression-lock test, surfacing the change.

## Annotation campaign
The systematic effort to add return annotations (`-> ...`) to grammar rules in `grammars/systemverilog.ebnf`. Each slice annotates one rule (or a small family) and bumps the schema version. The regex parser campaign (42+ slices) is the model for the SV campaign.

## ParseContent
The internal AST-pipeline representation used by PGEN's parser hooks and codegen. Variants include `Terminal`, `Sequence`, `Quantified`, `Or`, `Alternative`, `Json`. See [Parse Content Variants](parse-content-variants.md).

## Profile
A named configuration of the grammar that selects which top-level entry rule to start parsing from. SV profiles: `sv_2017`, `sv_2023`.

## Recursive envelope
The default JSON shape produced by un-annotated rules — a recursive composition of arrays (for sequences and iterations), strings (for terminals), and matched-branch passthroughs (for alternations). See [AST Envelope Structure](ast-envelope.md).

## Return annotation
A `-> ...` clause appended to a grammar rule definition that overrides the default envelope shape with a typed JSON value. Example: `simple_identifier -> {type: "identifier", name: $1}`. See `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Schema version
Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to. Currently `1`. See [Schema Versioning](schema-versioning.md).

## Slice
The unit of work in the annotation campaign. One slice = one rule (or a small family of related rules) annotated, the manifest updated, the contract bumped, the book amended, and a regression-lock test landed.

## Stable surface
The set of public entry points downstream consumers can rely on across patch / minor releases. Defined in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` § "Stable Integration Surface".

## Stress sample
A test input designed to exercise edge cases of the grammar — e.g. deeply nested structures, large modules, full LRM Annex-A coverage. Lives in `rust/test_data/grammar_quality/systemverilog_*` corpora.

## Typed shape
The JSON shape produced by an annotated rule. Always a JSON object with at least a `type` field and usually a `kind` discriminator. Consumers walking typed shapes dispatch on `obj["type"]` / `obj["kind"]`.
