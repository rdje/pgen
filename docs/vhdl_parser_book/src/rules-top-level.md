# Top-Level Rules

This chapter is the per-rule shape reference for the PGEN VHDL parser. It documents the `vhdl_file` root, the `design_unit` dispatch, and then enumerates the typed rule shapes grouped by rule family.

> **Status:** VHDL-Slice-1 (parser release `1.0.1`) landed the full grammar typing in one comprehensive batch; the `1.0.2` `VHDL-0001` correctness fix (AST-dump schema version `1` → `2`) added the two named operator rules. The current typed surface is **112 distinct rules / 256 return annotations** across `grammars/vhdl.ebnf` (parser release `1.0.2`, AST-dump schema version `2`). Unlike the SystemVerilog campaign, which types rules slice-by-slice, the VHDL grammar was annotated line-by-line in a single pass. Every shape in this chapter is drawn from the live inventory at `generated/vhdl_return_annotations.json` (cross-checked against the embedded inventory in `rust/test_data/ast_shape_contract/vhdl_v1.json` — identical content, 256 entries). That artifact, not this prose, is the machine-checkable source of truth.

## How to read this chapter

This is a **curated, grouped** reference — not a raw 256-line dump and not a copy of the IEEE 1076 LRM. For each family it gives the `kind` discriminators and field lists that the parser actually emits, as transcribed from each rule's normalized return-annotation text. Where a rule has per-branch typing, the `kind` value names the matched branch; where a rule has a single sequence shape, the named fields are listed directly.

Two conventions appear throughout:

- **Dispatch rules** emit `{kind: "<branch>", body: $N}` (or named fields per branch). Consumers dispatch on `obj["kind"]`.
- **`{first, rest}` list rules** emit a head element plus the raw iteration of the `(sep X)*` tail. This is the VHDL grammar's mini-mixed-array convention for separated lists (`identifier_list`, `association_list`, `selected_name`, etc.); the head is `first`, and `rest` is the trailing-iteration envelope.

The annotation-language conventions (`$N`, `{field: value}`, `[...]`, string literals) follow `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Entry point

| Profile | Entry rule | Description |
|---|---|---|
| `vhdl_1076_2019` | `vhdl_file` | IEEE 1076-2019 source file. The only stable VHDL profile. |

The host entry points are `parse_vhdl_1076_2019` / `parse_vhdl_1076_2019_ast_dump` (and the generic `parse_grammar_profile*` family with `GrammarFamily::Vhdl` + `GrammarProfile::Vhdl1076_2019`). See [Public API Surface](public-api.md).

## `vhdl_file` (root)

Per `grammars/vhdl.ebnf`:

```ebnf
vhdl_file := design_unit*
          -> {type: "vhdl_file", design_units: $1}
```

The annotation produces a typed JSON object at the root of every parse:

```json
{
  "type": "vhdl_file",
  "design_units": [/* array of design_unit shapes */]
}
```

`design_units` is the array of typed `design_unit` objects, one per top-level library unit in source order. Consumers walking the VHDL AST dispatch on `obj["type"] == "vhdl_file"` at the root, then iterate `obj["design_units"]`.

## `design_unit` dispatch

`design_unit` is the primary top-level dispatcher — a 10-branch `kind`-tagged shape covering every VHDL library-unit form:

```ebnf
design_unit := library_clause              -> {kind: "library",            body: $1}
             | use_clause                  -> {kind: "use",                body: $1}
             | context_reference_clause    -> {kind: "context_reference",  body: $1}
             | entity_declaration          -> {kind: "entity",             body: $1}
             | architecture_body           -> {kind: "architecture",       body: $1}
             | package_declaration         -> {kind: "package",            body: $1}
             | package_body                -> {kind: "package_body",       body: $1}
             | configuration_declaration   -> {kind: "configuration",      body: $1}
             | context_declaration         -> {kind: "context",            body: $1}
             | semi                        -> {kind: "semi"}
```

| `kind` | `body` shape |
|---|---|
| `"library"` | `library_clause` — `{first, rest}` |
| `"use"` | `use_clause` — `{first, rest}` |
| `"context_reference"` | `context_reference_clause` — `{name}` |
| `"entity"` | `entity_declaration` — `{name, items, end_label}` |
| `"architecture"` | `architecture_body` — `{name, entity_name, items, statements, end_label}` |
| `"package"` | `package_declaration` — `{name, header, items, end_label}` |
| `"package_body"` | `package_body` — `{name, items, end_label}` |
| `"configuration"` | `configuration_declaration` — `{name, entity_name, items, end_label}` |
| `"context"` | `context_declaration` — `{name, items, end_label}` |
| `"semi"` | _(no body — lone `;` separator)_ |

The `"semi"` branch carries only `{kind: "semi"}` (a stray top-level `;`); every other branch carries a `body`.

## Family: context clauses and selected names

| Rule | Shape |
|---|---|
| `library_clause` | `{first, rest}` — first identifier + comma-separated iteration. |
| `use_clause` | `{first, rest}` — first selected name + comma-separated iteration. |
| `context_reference_clause` | `{name}` — the referenced context name. |
| `context_item` (4 kinds) | `{kind, body}` for `"library"` / `"use"` / `"context_reference"`; `{kind: "semi"}` for the lone-`;` branch. |
| `selected_name` | `{first, rest}` — leading name segment + dotted-suffix iteration. |

## Family: design units

| Rule | Shape |
|---|---|
| `entity_declaration` | `{name, items, end_label}` — `items` is the `entity_declarative_item*` array; `end_label` is the optional trailing identifier after `end`. |
| `architecture_body` | `{name, entity_name, items, statements, end_label}` — `items` are declarative items, `statements` are concurrent statements. |
| `package_declaration` | `{name, header, items, end_label}` — `header` is the optional `package_header`. |
| `package_body` | `{name, items, end_label}`. |
| `configuration_declaration` | `{name, entity_name, items, end_label}` — `items` is the `configuration_item*` array. |
| `context_declaration` | `{name, items, end_label}` — `items` is the `context_item*` array. |
| `configuration_item` (3 kinds) | `{kind: "for", target}` / `{kind: "use", body}` / `{kind: "semi"}`. |

### Declarative-item dispatch rules

Each declarative region uses a `kind`-tagged dispatch over the declarations it admits. All emit `{kind, body}` except the bodyless `"semi"` branch (`{kind: "semi"}`).

| Rule | `kind` values |
|---|---|
| `entity_declarative_item` (5) | `"generic"`, `"port"`, `"component"`, `"use"`, `"semi"` |
| `architecture_declarative_item` (12) | `"signal"`, `"constant"`, `"file"`, `"type"`, `"subtype"`, `"component"`, `"subprogram_declaration"`, `"subprogram_body"`, `"package_instantiation"`, `"alias"`, `"use"`, `"semi"` |
| `package_declarative_item` (13) | `"subprogram_declaration"`, `"subprogram_body"`, `"type"`, `"subtype"`, `"constant"`, `"signal"`, `"file"`, `"component"`, `"package_instantiation"`, `"view"`, `"alias"`, `"use"`, `"semi"` |
| `package_body_declarative_item` (8) | `"subprogram_declaration"`, `"subprogram_body"`, `"constant"`, `"signal"`, `"file"`, `"alias"`, `"use"`, `"semi"` |
| `subprogram_declarative_item` (9) | `"variable"`, `"file"`, `"constant"`, `"type"`, `"subtype"`, `"subprogram_declaration"`, `"subprogram_body"`, `"use"`, `"semi"` |
| `process_declarative_item` (9) | `"variable"`, `"file"`, `"constant"`, `"type"`, `"subtype"`, `"subprogram_declaration"`, `"subprogram_body"`, `"use"`, `"semi"` |

## Family: generic / port / parameter interfaces

| Rule | Shape |
|---|---|
| `generic_clause` | `{list}` — the inner `generic_interface_list`. |
| `generic_interface_list` | `{first, rest}`. |
| `generic_interface_element` (2 kinds) | `{kind: "value", names, subtype, default}` / `{kind: "package", body}`. |
| `port_clause` | `{list}`. |
| `port_interface_list` | `{first, rest}`. |
| `port_interface_element` | `{names, mode, subtype, default}` — `mode` is the typed `signal_mode`. |
| `parameter_list` | `{first, rest}`. |
| `parameter_interface_element` | `{names, mode, subtype, default}` — `mode` is the typed `parameter_mode`. |
| `generic_map_aspect` | `{associations}` — the inner `association_list`. |
| `port_map_aspect` | `{associations}`. |
| `association_list` | `{first, rest}`. |
| `association_element` | `{formal, actual}`. |
| `actual_part` (2 kinds) | `{kind: "expression", body}` / `{kind: "open"}`. |
| `actual_parameter_part` | `{first, rest}`. |
| `actual_parameter_element` (2 kinds) | `{kind: "association", body}` / `{kind: "range_expression", body}`. |
| `identifier_list` | `{first, rest}`. |

### Mode keyword leaves

| Rule | `kind` values |
|---|---|
| `signal_mode` (5) | `"in"`, `"out"`, `"inout"`, `"buffer"`, `"linkage"` |
| `view_mode` (5) | `"in"`, `"out"`, `"inout"`, `"buffer"`, `"linkage"` |
| `parameter_mode` (3) | `"in"`, `"out"`, `"inout"` |

Each is a bare `{kind}` object (no body) — the keyword token is redundant with `kind`.

## Family: declarations

| Rule | Shape |
|---|---|
| `signal_declaration` | `{names, subtype, default}` — `default` is `[]` when no `:= expr` initializer. |
| `constant_declaration` | `{names, subtype, value}`. |
| `variable_declaration` | `{names, subtype, default}`. |
| `file_declaration` | `{names, subtype, open_mode, filename}`. |
| `subtype_declaration` | `{name, subtype}`. |
| `component_declaration` | `{name, generic, port, end_label}`. |
| `type_declaration` | `{name, definition}` — `definition` is the typed `type_definition`. |
| `view_declaration` | `{name, subtype, elements}`. |
| `view_element` | `{name, mode}` — `mode` is the typed `view_mode`. |
| `alias_declaration` | `{name, target}`. |
| `package_header` | `{generic}`. |
| `package_generic_interface_element` | `{name, base, map}`. |
| `package_instantiation_declaration` | `{name, base, map}`. |
| `subprogram_declaration` (2 kinds) | `{kind: "procedure", spec}` / `{kind: "function", spec}`. |
| `subprogram_body` (2 kinds) | `{kind: "procedure", spec, items, statements, end_label}` / `{kind: "function", spec, items, statements, end_label}`. |
| `procedure_specification` | `{name, parameters}`. |
| `function_specification` | `{impure, name, parameters, return_type}` — `impure` is `[]` for a pure function, set when `impure` is present. |

## Family: types and constraints

| Rule | Shape |
|---|---|
| `type_definition` (3 kinds) | `{kind: "enumeration", body}` / `{kind: "array", body}` / `{kind: "record", body}`. |
| `enumeration_type_definition` | `{first, rest}`. |
| `array_type_definition` | `{index_range, element_subtype}`. |
| `record_type_definition` | `{elements}`. |
| `record_element_declaration` | `{names, subtype}`. |
| `subtype_indication` | `{type_mark, constraint}` — `constraint` is `[]` when the subtype is unconstrained. |
| `constraint` (2 kinds) | `{kind: "range", body}` / `{kind: "index", body}`. |
| `range_constraint` | `{range}`. |
| `index_constraint` | `{first, rest}`. |
| `discrete_range` (5 kinds) | `{kind: "range", body}` / `{kind: "attribute", body}` / `{kind: "name", body}` / `{kind: "subtype_range", subtype, range}` / `{kind: "subtype_box", subtype}`. |
| `range_expression` (2 kinds) | `{kind: "to", low, high}` / `{kind: "downto", high, low}`. |

## Family: concurrent statements

| Rule | Shape |
|---|---|
| `concurrent_statement` (7 kinds) | `{kind, body}` for `"signal_assignment"` / `"assert"` / `"process"` / `"component_instantiation"` / `"generate"` / `"block"`; `{kind: "semi"}` for the lone-`;` branch. |
| `concurrent_signal_assignment_statement` | `{target, rhs}`. |
| `process_statement` | `{label, sensitivity, items, statements, end_label}`. |
| `process_label` | `{name}`. |
| `sensitivity_clause` | `{list}` — the inner `sensitivity_list`. |
| `sensitivity_list` | `{first, rest}`. |
| `component_instantiation_statement` | `{label, unit, generic_map, port_map}` — `unit` is the typed `instantiated_unit`. |
| `instantiated_unit` (3 kinds) | `{kind: "component", name}` / `{kind: "entity", name}` / `{kind: "configuration", name}`. |
| `generate_statement` (2 kinds) | `{kind: "for", label, var, range, statements}` / `{kind: "if", label, condition, statements}`. |
| `block_statement` | `{label, items, statements, end_label}`. |

## Family: sequential statements

| Rule | Shape |
|---|---|
| `sequential_statement` (13 kinds) | `{kind, body}` for `"signal_assignment"` / `"variable_assignment"` / `"if"` / `"case"` / `"loop"` / `"exit"` / `"wait"` / `"assert"` / `"report"` / `"return"` / `"procedure_call"`; `{kind: "null"}` and `{kind: "semi"}` are bodyless. |
| `signal_assignment_statement` | `{target, rhs}`. |
| `signal_assignment_rhs` | `{value, conditional}` — `conditional` carries the optional `when`-clause tail. |
| `variable_assignment_statement` | `{target, value}`. |
| `target` (2 kinds) | `{kind: "name", name, params}` / `{kind: "aggregate", first, rest}`. |
| `if_statement` | `{condition, then_body, elsif_branches, else_body}` — `elsif_branches` is the `(elsif … then …)*` iteration; `else_body` is `[]` when there is no `else`. |
| `case_statement` | `{value, alternatives}`. |
| `case_statement_alternative` | `{choices, body}`. |
| `loop_statement` (3 kinds) | `{kind: "while", condition, body}` / `{kind: "for", var, range, body}` / `{kind: "infinite", body}`. |
| `exit_statement` | `{condition_or_label}`. |
| `wait_statement` | `{timeout}` — the optional `for expression` clause. |
| `assert_statement` | `{condition, report, severity}`. |
| `report_statement` | `{message, severity}`. |
| `return_statement` | `{value}` — `value` is `[]` for a bare `return;`. |
| `procedure_call_statement` | `{name, params}`. |

## Family: expressions — the `binop_chain` contract

VHDL's expression grammar is a five-level operator-precedence cascade. Each level carries a `binop_chain` typed shape:

```ebnf
expression        := relation (logical_operator relation)*
                  -> {type: "binop_chain", level: "logical",         lhs: $1, rest: $2}
relation          := simple_expression (relational_operator simple_expression)?
                  -> {type: "binop_chain", level: "relational",      lhs: $1, rest: $2}
simple_expression := (plus | minus)? term (adding_operator term)*
                  -> {type: "binop_chain", level: "additive",        sign: $1, lhs: $2, rest: $3}
term              := factor (multiplying_operator factor)*
                  -> {type: "binop_chain", level: "multiplicative",  lhs: $1, rest: $2}
factor            := primary (power primary)?
                  -> {type: "binop_chain", level: "power",           lhs: $1, rest: $2}
```

> **Schema `2` note (`VHDL-0001`).** The `adding_operator` /
> `multiplying_operator` iteration leads are **named rules** as of the
> `1.0.2` correctness fix. At `1.0.1` / schema `1` they were inline
> alternations (`(plus | minus | ampersand)` / `(star | slash | kw_mod
> | kw_rem)`), which corrupted the positional model so the `additive`
> (`simple_expression`) and `multiplicative` (`term`)
> `binop_chain.rest` emitted `"<invalid_sequence_access>"` on
> multi-operand input — a real parser defect fixed in `1.0.2` / schema
> `2`. The `simple_expression` / `term` `binop_chain` annotations are
> unchanged (only the inline group became a named rule); the leading
> `(plus | minus)?` `sign` is not an iteration lead and was empirically
> unaffected — left as-is. See [Schema Versioning](schema-versioning.md)
> and the [Binary Addition](examples-binary-addition.md) worked example.

| Level (`level`) | Rule | Fields | Operators |
|---|---|---|---|
| `"logical"` | `expression` | `lhs`, `rest` | `and` / `or` / `xor` / `nand` / `nor` / `xnor` (`logical_operator`) |
| `"relational"` | `relation` | `lhs`, `rest` | `=` / `/=` / `<` / `<=` / `>` / `>=` (`relational_operator`) |
| `"additive"` | `simple_expression` | `sign`, `lhs`, `rest` | unary `+`/`-` (`sign`), binary `+` / `-` / `&` (`adding_operator`) |
| `"multiplicative"` | `term` | `lhs`, `rest` | `*` / `/` / `mod` / `rem` (`multiplying_operator`) |
| `"power"` | `factor` | `lhs`, `rest` | `**` |

**Consumer-facing left-fold contract** (per the integration contract, Release 1.0.2 Highlights): every level emits `{type: "binop_chain", level, lhs, rest}` where `lhs` is the leading operand and `rest` is the clean iteration array of `[op-envelope, operand]` entries. At **every** level the op-envelope is the typed `{kind: …}` object (uniform — `logical_operator` / `relational_operator` / `adding_operator` / `multiplying_operator` all emit `{kind}`); no level ever emits `<invalid_sequence_access>` as of schema `2`. **Consumers fold `rest` left-associatively onto `lhs`**, reading the operator from `op-envelope.kind`. At the `"additive"` level there is an additional leading `sign` field carrying the optional unary `+`/`-` (`[]` when absent). Two levels cap `rest` at **at most one** entry because their grammar rule uses `?` (not `*`): `"relational"` (`relation := simple_expression (relational_operator simple_expression)?`) and `"power"` (`factor := primary (power primary)?`). The other three levels — `"logical"` (`expression`), `"additive"` (`simple_expression`), and `"multiplicative"` (`term`) — iterate `*`, so their `rest` is a 0..N array. A correct left-fold consumer handles both uniformly (fold over a `rest` of length 0, 1, or N).

This `binop_chain` shape is the same across all five levels precisely so that a single consumer fold routine handles the entire expression tree. See [Walking the AST](walking-the-ast.md) for a worked fold and the [Binary Addition](examples-binary-addition.md) worked example for the real captured `b + c * d` shape.

### Operators (bare `{kind}` leaves)

All four operator rules are typed bare-`{kind}` leaves (no `body`); the op-envelope in `binop_chain.rest` at every level is the typed `{kind}` object these rules emit (schema `2`). `adding_operator` / `multiplying_operator` are the `1.0.2` `VHDL-0001` named-rule fix (they were inline alternations at `1.0.1`).

| Rule | `kind` values |
|---|---|
| `logical_operator` (6) | `"and"`, `"or"`, `"xor"`, `"nand"`, `"nor"`, `"xnor"` |
| `relational_operator` (6) | `"eq"`, `"ne"`, `"lt"`, `"le"`, `"gt"`, `"ge"` |
| `adding_operator` (3) — **`1.0.2` `VHDL-0001` fix** | `"plus"`, `"minus"`, `"concat"` |
| `multiplying_operator` (4) — **`1.0.2` `VHDL-0001` fix** | `"mul"`, `"div"`, `"mod"`, `"rem"` |

The 3 `adding_operator` + 4 `multiplying_operator` `return_object` branches are the +7 entries that took the annotation count `249 → 256` and the distinct-rule count `110 → 112` at the `1.0.2` fix. (`factor`'s `power` operator is still matched inline as a literal token within `rest`, not via a separate typed operator rule — only `**` at one level.)

### `primary` and aggregates

| Rule | Shape |
|---|---|
| `primary` (7 kinds) | `{kind: "literal", body}` / `{kind: "aggregate", body}` / `{kind: "attribute_name", body}` / `{kind: "function_call", name, params}` / `{kind: "name", body}` / `{kind: "parens", expr}` / `{kind: "not", expr}`. |
| `attribute_name` | `{prefix, prefix_params, attribute, attribute_params}`. |
| `aggregate` (2 kinds) | `{kind: "named_first", first_choices, first_value, rest}` / `{kind: "positional_first", first_value, second, rest}`. |
| `aggregate_element_association` (2 kinds) | `{kind: "named", choices, value}` / `{kind: "positional", value}`. |
| `aggregate_choice_list` | `{first, rest}`. |
| `aggregate_choice` (5 kinds) | `{kind: "others"}` / `{kind: "name", body}` / `{kind: "decimal", body}` / `{kind: "character", body}` / `{kind: "string", body}`. |
| `choices` | `{first, rest}`. |
| `choice` (2 kinds) | `{kind: "expression", body}` / `{kind: "others"}`. |

## Family: literals

| Rule | Shape |
|---|---|
| `literal` (6 kinds) | `{kind, body}` for `"physical"` / `"decimal"` / `"based"` / `"bit_string"` / `"string"` / `"character"`. |
| `decimal_literal` | `{value}`. |
| `based_literal` | `{base, value}`. |

`physical_literal`, `bit_string_literal`, `string_literal`, and `character_literal` are matched as terminal/regex leaves and carried through the recursive-envelope shape (string of matched text); they have no per-rule annotation — only the dispatch in `literal` is typed.

## Total surface and the machine-checkable source

The full typed surface as of contract `1.0.2` is **256 annotations across 112 distinct rules**. This chapter is a curated grouping; the authoritative, machine-checkable enumeration of every `(rule, branch_index, annotation_type, normalized_text)` tuple is:

- `generated/vhdl_return_annotations.json` — the live return-annotation inventory (inventory-file `version: 1` — the inventory format version, not the AST-dump schema; `grammar: "vhdl"`, `annotation_count: 256`).
- `rust/test_data/ast_shape_contract/vhdl_v1.json` — the embedded inventory used by the AST shape-contract regression lock (identical content; 256 entries).

If this chapter and either artifact disagree, the artifact wins — and the integration contract `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` wins over both.

## How to follow per-slice changes

Each shape-affecting slice after VHDL-Slice-1 gets a row in [Schema Versioning](schema-versioning.md) and a Highlights section in `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`. The [Changelog Index](changelog-index.md) ties them together.
