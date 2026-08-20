# LRM clause-only productions — the Annex A gap census

> DERIVED — regenerate with `python3 stimuli/sv/lrm_annex_a_gap_census.py --write`.
> Owning leaf: `SV-CORPUS-GRAD.13c.2v`. Never hand-edit.

`grammars/systemverilog.ebnf` descends from **Annex A alone**. IEEE 1800 also defines
productions in its clause bodies, and its own syntax-box captions say which:
`(excerpt from Annex A)` versus **`(not in Annex A)`**. Every row below is a production
the clause text defines that Annex A does not carry.

⚠️ **`modelled in shipped grammar` is a NAME lookup, and it reads `0` everywhere by
construction** — the grammar is extracted from Annex A, so it cannot carry a rule named
after a production Annex A does not have. It is **not** a coverage verdict: a clause-only
production is routinely *reachable* through a general Annex A rule (clause 20/21's system
tasks through `system_tf_call`, for one). Coverage is decided per row by a witness —
`worklist_probes/probe_worklist.sh` — never by this column.

## IEEE 1800-2017

- clause-only productions: **147**

| bucket | rows | IEEE says `not in Annex A` | modelled in shipped grammar |
|---|---:|---:|---:|
| `compiler_directive` | 23 | 23 | 0 |
| `formal_semantics_metavariable` | 7 | 7 | 0 |
| `sv_source_syntax` | 7 | 6 | 0 |
| `system_task_function` | 110 | 110 | 0 |

### `sv_source_syntax` — the adjudication worklist (7 rows)

| rule | IEEE caption | says | modelled | body |
|---|---|---|---|---|
| `array_method_call` | Syntax 7-5 | not in Annex A | no | `expression . array_method_name { attribute_instance } [ ( iterator_argument ) ] [ with ( expression ) ]` |
| `built_in_data_type` | Syntax 26-5 | not in Annex A | no | `[ std :: ] data_type_identifier` |
| `built_in_function_call` | Syntax 26-5 | not in Annex A | no | `[ std :: ] function_subroutine_call` |
| `item_name` | Syntax 23-8 | not in Annex A | no | `function_identifier | block_identifier | net_identifier | parameter_identifier | port_identifier | task_identi` |
| `scope_randomize` | Syntax 18-11 | not in Annex A | no | `[ std :: ] randomize ( [ variable_identifier_list ] ) [ with constraint_block ]` |
| `solve_before_primary` | Syntax 18-8 | excerpt from Annex A | no | `[ implicit_class_handle . | class_scope ] hierarchical_identifier select` |
| `upward_name_reference` | Syntax 23-8 | not in Annex A | no | `module_identifier.item_name` |

## IEEE 1800-2023

- clause-only productions: **147**

| bucket | rows | IEEE says `not in Annex A` | modelled in shipped grammar |
|---|---:|---:|---:|
| `compiler_directive` | 24 | 24 | 0 |
| `formal_semantics_metavariable` | 7 | 7 | 0 |
| `sv_source_syntax` | 7 | 7 | 0 |
| `system_task_function` | 109 | 109 | 0 |

### `sv_source_syntax` — the adjudication worklist (7 rows)

| rule | IEEE caption | says | modelled | body |
|---|---|---|---|---|
| `array_method_call` | Syntax 7-5 | not in Annex A | no | `expression . array_method_name { attribute_instance } [ ( [ iterator_argument ] [ , index_argument ] ) ] [ wit` |
| `built_in_data_type` | Syntax 26-5 | not in Annex A | no | `[ std :: ] data_type_identifier` |
| `built_in_function_call` | Syntax 26-5 | not in Annex A | no | `[ std :: ] function_subroutine_call` |
| `inline_constraint_declaration` | Syntax 18-10 | not in Annex A | no | `// not in Annex A class_variable_identifier . randomize [ ( [ variable_identifier_list | null ] ) ] with [ ( [` |
| `item_name` | Syntax 23-8 | not in Annex A | no | `function_identifier | block_identifier | net_identifier | parameter_identifier | port_identifier | task_identi` |
| `scope_randomize` | Syntax 18-11 | not in Annex A | no | `[ std :: ] randomize ( [ variable_identifier_list ] ) [ with constraint_block ]` |
| `upward_name_reference` | Syntax 23-8 | not in Annex A | no | `module_identifier.item_name` |

