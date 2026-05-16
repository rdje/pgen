# Glossary

Terms used throughout this book. Where a term has a normative definition, the integration contract `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` is authoritative; this glossary paraphrases it for quick lookup.

## AST envelope

The JSON tree returned by the rtl_const_expr AST-dump entry points. It is the `root` field of the `AstDumpPayload` carried by `GrammarAstDumpOutcome`. Every parse roots at the single typed object `{type: "rtl_const_expr", expr: <conditional_expr-shape>}`. See [AST Envelope Structure](ast-envelope.md).

## AST shape contract manifest

The file `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` (`version: 1`, `grammar: "rtl_const_expr"`). It records the per-rule expected JSON shape for each sample in the rtl_const_expr test corpus (the `literal_42` and `binary_addition` samples) and embeds the declared-annotation inventory — **26 annotation entries** as of contract `1.0.2`. Drift in the AST dump fails the `rtl_const_expr_ast_shape_contract` regression-lock test under `cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract`, surfacing the change. Its `declared_annotation_inventory.annotations` list is content-identical to the live inventory `generated/rtl_const_expr_return_annotations.json` — the `(rule, branch_index, annotation_type, normalized_text)` tuples match exactly; the live inventory additionally carries a per-entry `raw_text` field. (This is the rtl_const_expr manifest; the SystemVerilog, VHDL, and rtl_frontend parsers have their own separate `systemverilog_v1.json` / `vhdl_v1.json` / `rtl_frontend_v1.json`.)

## binop_chain

The consumer-facing left-fold contract for rtl_const_expr's expression-precedence hierarchy. rtl_const_expr has a **ten-level** binary-operator cascade — `logical_or_expr` → `logical_and_expr` → `bit_or_expr` → `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr` → `shift_expr` → `additive_expr` → `multiplicative_expr` (independently counted: exactly **10** of the 18 typed rules emit `{type: "binop_chain", ...}`). Every level emits the same `{type: "binop_chain", level, lhs, rest}` shape, so a single consumer fold handles the whole expression tree. `lhs` (`$1`) is the leading operand (itself a `binop_chain` of the next-tighter level, bottoming out at a typed `primary_expr` leaf via `unary_expr`); `rest` (`$2`) is a **clean array** of `[ <op-envelope>, <operand> ]` iteration entries from the `( <op> <next> )*` tail, folded left-associatively, or the empty array `[]` when there is no operator at that level. The operator text is reached at `entry[0][1]`; the operand is `entry[1]`. There is **no** `sign` field on any rtl_const_expr level (unlike VHDL's `simple_expression`) — prefix `+` / `-` / `!` / `~` live in the `unary_expr` rule below the cascade. As of parser release `1.0.2` the `rest` array is never `"<invalid_sequence_access>"` (see [Schema Versioning](schema-versioning.md)). See [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract) for the level/field/operator table and [Walking the AST](walking-the-ast.md#folding-the-binop_chain-expression-hierarchy) for the fold code.

## Declared-annotation inventory

The machine-checkable enumeration of every typed-shape annotation the rtl_const_expr grammar emits: `generated/rtl_const_expr_return_annotations.json` (`version: 1`, `grammar: "rtl_const_expr"`, `annotation_count: 26`). It is the live source of truth for the typed surface and is mirrored content-for-content by the embedded inventory in `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` (26 entries). Of the 26 entries, **19** are `return_object` annotations and **7** are `return_scalar`; the 26 are spread across **18 distinct rules**. If this book's prose disagrees with the inventory, the inventory wins; if the inventory disagrees with the integration contract, the contract wins.

## Named operator rules

The five **un-annotated** alternation rules introduced by the `1.0.2` correctness fix to lift each multi-token operator out of its `binop_chain` tail so the iteration captures cleanly:

```ebnf
equality_op       := eqeq | ne
relational_op     := le | lt | ge | gt
shift_op          := shl | shr
additive_op       := plus | minus
multiplicative_op := star | slash | percent
```

These rules carry **no** return annotation and are therefore **not** part of the 26-entry inventory. They exist purely so that each affected level reads `next ( NAMED_op next )* -> {type: "binop_chain", level, lhs: $1, rest: $2}` with a bare `$2`, mirroring the proven `systemverilog.ebnf` op-chain idiom. The other five single-token levels (`logical_or`, `logical_and`, `bit_or`, `bit_xor`, `bit_and`) already used a single named token rule and were unchanged. See [Top-Level Rules](rules-top-level.md#family-expressions--the-binop_chain-contract) and [Schema Versioning](schema-versioning.md).

## `<invalid_sequence_access>` (historical)

The literal string the pre-`1.0.2` parser emitted inside a malformed nested object for the `binop_chain` `rest` field on any input exercising an operator at any of the ten levels. It was a return-annotation defect (an inline operator alternation used as an iteration's lead element corrupted the positional model), corrected in parser release `1.0.2` by lifting those alternations into the **named operator rules**. A `1.0.2`-or-later parser never emits this string; observing it indicates a stale (schema-`1`) parser. See the schema-`2` row in [Schema Versioning](schema-versioning.md).

## parseability_probe

The CLI wrapper around `pgen::embedding_api` used for terminal-side verification, AST inspection, and bug-report reproducers. Sub-commands include `--parse`, `--parse-dump-ast`, and `--parse-dump-ast-pretty`. For rtl_const_expr the parser is on-demand-only, so the probe must be built with the generated backend before use (see [Build Recipe](build-recipe.md)). The full flag set, exit codes, and registered grammars are in the [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md).

## Parser release version

The parser library's release identity, currently `1.0.2`. Bumped on every functional change to the parser, including bug fixes, performance work, and grammar changes. It moves independently of the schema version: a release can carry the same schema version as the previous one (no shape change) or a bumped one (shape changed) — the `1.0.2` correctness fix bumped both (release `1.0.1` → `1.0.2`, schema `1` → `2`). Recorded in `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity". See [Schema Versioning](schema-versioning.md).

## Profile

A named configuration of the grammar that selects which top-level entry rule to start parsing from. rtl_const_expr has exactly **one** profile: `default`, whose entry rule is the grammar root `rtl_const_expr` (the synthesizable RTL constant-expression). There is **no** per-grammar convenience function for rtl_const_expr (no `parse_rtl_const_expr`); the stable host surface is the generic-by-grammar entry points — `parse_grammar_profile`, `parse_grammar_profile_result`, `parse_grammar_profile_ast_dump` with `GrammarFamily::RtlConstExpr` + `GrammarProfile::Default`, plus the `parse_grammar_profile_named` string overload with `"rtl_const_expr"` / `"default"`. See [Public API Surface](public-api.md).

## Recursive envelope

The default JSON shape produced by un-annotated rules — a recursive composition of arrays (for sequences and the `( <op> <next> )*` quantified iteration that forms a `binop_chain`'s `rest` tail), strings (for terminal and regex leaves), and matched-branch passthroughs (for alternations). An empty `*` iteration is the empty array `[]`, never `null`. In rtl_const_expr the recursive envelope is what you reach in exactly two places: the `rest` field of every `binop_chain` (a clean `[ <op-envelope>, <operand> ]` array since `1.0.2`), and the un-annotated leaves — the keyword/operator tokens, the five named operator alternation rules, and `trivia`. See [AST Envelope Structure](ast-envelope.md) and [The Json Carrier](json-carrier.md).

## Return annotation

A `-> ...` clause appended to a grammar rule definition in `grammars/rtl_const_expr.ebnf` that overrides the default recursive-envelope shape with a typed JSON value. Example: `rtl_const_expr := conditional_expr -> {type: "rtl_const_expr", expr: $1}`. The annotation language (`$N` positional references, `{field: value}` object literals, string literals) is specified in `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`. rtl_const_expr uses two annotation kinds: `return_object` (an object literal; 19 of the 26) and `return_scalar` (a positional reference; 7 of the 26).

## return_object / return_scalar

The two `annotation_type` values in the inventory. **`return_object`** (19 entries) is an object literal — the root, `conditional_expr` branch 0, the ten `binop_chain` levels, `unary_expr` branches 0–3, and `literal`'s two branches. **`return_scalar`** (7 entries) is a bare positional reference and takes two roles: five are **passthrough** branches that contribute no wrapper and surface the matched sub-rule's shape directly (`conditional_expr` branch 1, `primary_expr` branches 0/1/2, `unary_expr` branch 4), and two are leaf **scalar captures** (`based_integer` / `decimal_integer`, both `-> $2`) that bind the matched regex span to a clean string feeding `literal.text`. See [The Json Carrier](json-carrier.md) and [Top-Level Rules](rules-top-level.md).

## Schema version

Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to (a new annotation on a previously-unannotated rule, a restructured annotation, a user-visible grammar-shape change). Pure performance work and internal codegen reorganization do **not** bump it. The integer `AstDumpPayload.schema_version` field consumers branch on at runtime is currently `2` (the `1.0.2` correctness fix bumped it from `1`). The contract's "Schema Versioning" table additionally uses `1.0.2` / `1.0.0` / `0.1.0` milestone labels for the typing-campaign timeline. See [Schema Versioning](schema-versioning.md).

## Typed shape

The JSON value produced by an annotated rule. In rtl_const_expr it is uniformly a `type`-tagged object — the root (`{type: "rtl_const_expr", expr}`), the ten `binop_chain` levels (`{type: "binop_chain", level, lhs, rest}`), `conditional_expr` branch 0 (`{type: "ternary", condition, then_expr, else_expr}`), `unary_expr` branches 0–3 (`{type: "unary", op, expr}`), `literal` (`{type: "literal", kind, text}`), and `identifier` (`{type: "identifier", text}`). There is **no** bare `kind` dispatcher anywhere in this grammar; `kind` / `level` / `op` are secondary discriminators **inside** a `type`-tagged object. Consumers dispatch on `obj["type"]`. See [The Json Carrier](json-carrier.md) and [Top-Level Rules](rules-top-level.md).

## Un-annotated leaf (rule of thumb)

A value reached through an un-annotated rule surfaces as that rule's **recursive envelope**, not a bare JSON string. In rtl_const_expr source text is surfaced as a clean string only in the `text` field of a `literal` or `identifier` object, and in `based_integer` / `decimal_integer`'s `-> $2` capture (which feeds `literal.text`) — those rules *are* annotated. Do **not** assume any other slot is a scalar string: in particular the operator token inside a `binop_chain` `rest` entry is the recursive envelope of an un-annotated named operator rule (for a `trivia "<tok>"` token this is `["", "<tok>"]`). The robust consumer rule is to walk to the terminal string (`entry[0][1]`, or `extract_terminal_text`) rather than indexing a fixed depth. See [The Json Carrier](json-carrier.md) and the worked walkthrough in [Walking the AST](walking-the-ast.md#identifier-and-literal-extraction).

## RTL-CE-Slice-1

The single comprehensive typing slice (landed 2026-05-14, parser release `1.0.1`, schema version `1`) that typed the entire `grammars/rtl_const_expr.ebnf` expression surface at once — conditional head, the ten-rule `binop_chain` hierarchy, the unary prefix tier, the primary tier, `literal` (two kinds), and `identifier`. Unlike the SystemVerilog and regex parsers, whose return annotations were added rule-by-rule over a long per-slice campaign (each slice bumping the schema version), rtl_const_expr was typed in one pass. The initial slice shipped a 24-annotation baseline whose `binop_chain` `rest`, `identifier.text`, and `literal.text` shapes were defective; the **RTL-CE-Slice-2** correctness fix corrected them (see below). This single-batch-then-correction history is why the rtl_const_expr [Changelog Index](changelog-index.md) and [Schema Versioning](schema-versioning.md) timeline are short by design.

## RTL-CE-Slice-2

The follow-up correctness fix (landed 2026-05-16, parser release `1.0.2`, schema version `2`) that corrected the three return-annotation defects shipped by RTL-CE-Slice-1: (a) the ten `binop_chain` levels no longer emit `"<invalid_sequence_access>"` — the five multi-token inner operator alternations were lifted into the **named operator rules** and each level's `rest` is now a clean `[ <op-envelope>, <operand> ]` array via bare `rest: $2`; (b) `identifier.text` was `$1` (the empty leading `trivia`) → `$2`, the real name; (c) `based_integer` / `decimal_integer` were unannotated (surfacing the `["", "42"]` envelope) → annotated `-> $2`, so `literal.text` is a clean string. Annotation inventory `24 → 26` (the two new leaf scalar captures). Same accept set — purely annotation shaping. Subsequent shape-affecting slices, if any, each get their own contract row and changelog entry.
