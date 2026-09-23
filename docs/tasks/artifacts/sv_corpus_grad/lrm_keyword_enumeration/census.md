# LRM keyword-enumeration membership census — the `net_type` class

Derived by `stimuli/sv/lrm_keyword_enumeration_census.py`; do not hand-edit. Re-run it
in gate mode and diff.

**The question.** For every LRM production whose alternatives are all bare tokens and
that names at least one reserved keyword — `net_type ::= supply0 | supply1 | … | wor`
— does the shipped grammar's rule for that production still reach EVERY keyword
member under the profile that edition is released as? `net_type` shipped ten of
twelve for the whole life of the flattened grammar (`SV-CORPUS-GRAD.13n`, ledger
`SV-0075`) and nothing structural noticed: every name resolved, every existing rule
was witnessed, and the corpus only saw it where a vendored file spelled `tri0`.

## Instrument identity

Re-hash these inputs; if any hash differs from the row below, **this report no longer
describes your tree** and the honest act is to re-measure, not to quote.

| input | sha256 |
| --- | --- |
| `grammars/systemverilog.ebnf` (ebnf_raw_ast) | `3a94c6de9e3229ec26ca0a0302d126c28e4329b7bbc418848bf5851fecc1b295` |
| `docs/systemverilog/2017/md/section-41-data-read-api.md` (Annex A + Annex B, 1800-2017) | `249ab571c2ef3237790a6bd9743f07a7595bdded337ca2cbb6d5e189b3b40d22` |
| `grammars/systemverilog_2017_lrm_extracted.ebnf` (snapshot, 1800-2017) | `17bf25cdaff3bf0b347a0188a62338b27eea08423623c33cc789af21056e8abe` |
| `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md` (Annex A, 1800-2023) | `0b4f7fe935e9c499a56ccde8dc769ed626bf53d154f8a9585bd1daf371a9d4aa` |
| `docs/systemverilog/2023/md/section-Annex_B-normative-keywords.md` (Annex B, 1800-2023) | `e4fd8a92572c3afdb87dc994efd08a4bfba8ef122cf562a7520dbe996fc66efd` |
| `grammars/systemverilog_2023_lrm_extracted.ebnf` (snapshot, 1800-2023) | `c28f5d106cb28277109d99be28247217f05ad36134bb9501d4af1f4f16d82a16` |
| `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md` (Annex A, 1364-2005) | `cf248228161538bde5a760cec61929d21bebd611ddd1229197e02559a1574754` |
| `docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md` (Annex B, 1364-2005) | `08ba9745b0f0791ad48b786284b733414ab26ce7b2b9373d95174d094218025d` |
| `grammars/verilog_2005_lrm_extracted.ebnf` (snapshot, 1364-2005) | `f54a577ecf96e52dbc2cea966ab846e56c7d11741161c47473459a0190cf80bc` |

## Headline

```text
LRM-KEYWORD-ENUMERATION: editions=3 productions=109 matched=100 unmatched=9 keyword_members=319 present=319 missing=0 missing_no_terminal=0 incomplete_productions=0 extra_members=8 literal_members=3
```

| edition | judged under | productions in population | matched by name | unmatched | keyword members | present | MISSING | extra (informational) |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 1800-2017 | `sv_2017` | 43 | 42 | 1 | 129 | 129 | **0** | 3 |
| 1800-2023 | `sv_2023` | 44 | 43 | 1 | 130 | 130 | **0** | 3 |
| 1364-2005 | `verilog_2005` | 22 | 15 | 7 | 60 | 60 | **0** | 2 |

## Every MISSING member — the rows to adjudicate

`MISSING` = a `kw_*` terminal spells the word but the rule named for this production
cannot reach it under the profile. `MISSING_NO_TERMINAL` = no terminal spells it at
all. ⛔ Text-level triage, not a parse verdict: each row is settled by the release
probe with a two-sided bisect (the input rejects; substituting a sibling member
accepts), exactly as `.13k`(b) settled `tri0`.

| edition | production | LRM citation | member | verdict | shipped rule(s) consulted |
|---|---|---|---|---|---|
| _(none)_ | | | | | |

## Unmatched productions — no shipped rule by name or profile variant (NOT scored)

The shipped grammar is a hand-annotated flattened synthesis, so a production the LRM
names may be lifted, renamed or folded into a neighbour. These rows are an
ADJUDICATION worklist: each needs a reader to name the shipped rule that carries the
enumeration (or to show the members are carried some other way) before a verdict
exists. They are listed so the bound is visible, not to be counted as defects.

| edition | production | LRM citation | kind | members |
|---|---|---|---|---|
| 1800-2017 | `scope_type` | `grammars/systemverilog_2017_lrm_extracted.ebnf:964` | pure | `module` |
| 1800-2023 | `scope_type` | `grammars/systemverilog_2023_lrm_extracted.ebnf:1105` | pure | `module` |
| 1364-2005 | `file_size` | `grammars/verilog_2005_lrm_extracted.ebnf:404` | pure | `integer` |
| 1364-2005 | `logic` | `grammars/verilog_2005_lrm_extracted.ebnf:710` | pure | `and` `or` `nand` `nor` |
| 1364-2005 | `output_variable_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:221` | pure | `integer` `time` |
| 1364-2005 | `parameter_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:161` | pure | `integer` `real` `realtime` `time` |
| 1364-2005 | `scope_type` | `grammars/verilog_2005_lrm_extracted.ebnf:1100` | pure | `begin` `fork` `function` `module` `task` |
| 1364-2005 | `task_port_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:379` | pure | `integer` `real` `realtime` `time` |
| 1364-2005 | `var_type` | `grammars/verilog_2005_lrm_extracted.ebnf:1445` | pure | `event` `integer` `parameter` `real` `realtime` `reg` `supply0` `supply1` `time` `tri` `triand` `trior` `trireg` `tri0` `tri1` `wand` `wire` `wor` |

## Extra members — the shipped closure reaches a keyword the LRM enumeration does not name (informational, NOT scored)

⛔ Deliberately not a verdict. *The annex does not derive it* is a KNOWN-BAD classifier
for over-acceptance (`LRM-GRAMMAR-FIDELITY.1c`: three constructs flagged that way were
all legal), the closure follows the shipped grammar's own profile-variant folding, and
the `verilog_2005` keyword-admission direction is owned by
`stimuli/sv/v2005_keyword_faithfulness_census.py` (TOOLBOX 5.11), which adjudicates
each candidate on the release probe with three legs.

| edition | production | extra members in the shipped closure |
|---|---|---|
| 1800-2017 | `assignment_pattern_key` | `chandle` `event` `string` |
| 1800-2023 | `assignment_pattern_key` | `chandle` `event` `string` |
| 1364-2005 | `net_type` | `trireg` |
| 1364-2005 | `timing_check_event_control` | `edge` |

## Every production in the population

`sources` says which LRM surface carried the enumeration: `annex_a` (the markdown the
shipped grammar descends from), `snapshot` (the tracked extraction snapshot), or both.
`literal` members are bare words the LRM writes that are neither keywords nor
productions (`none` in `default_nettype_value`); they are listed, never scored.

| edition | production | LRM citation | sources | kind | status | keyword members | literal members | shipped rule(s) |
|---|---|---|---|---|---|---:|---|---|
| 1800-2017 | `always_keyword` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2021` | annex_a+snapshot | pure | complete | 4 |  | `always_keyword` `always_keyword_sv_only` |
| 1800-2017 | `array_method_name` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2969` | annex_a | mixed | complete | 4 |  | `array_method_name` |
| 1800-2017 | `assignment_pattern_key` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2299` | annex_a+snapshot | mixed | complete | 1 |  | `assignment_pattern_key` |
| 1800-2017 | `bins_keyword` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1573` | annex_a+snapshot | pure | complete | 3 |  | `bins_keyword` |
| 1800-2017 | `case_keyword` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2238` | annex_a+snapshot | pure | complete | 3 |  | `case_keyword` |
| 1800-2017 | `casting_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:841` | annex_a+snapshot | mixed | complete | 2 |  | `casting_type` |
| 1800-2017 | `checker_port_direction` | `docs/systemverilog/2017/md/section-41-data-read-api.md:508` | annex_a | pure | complete | 2 |  | `checker_port_direction` |
| 1800-2017 | `class_item_qualifier` | `docs/systemverilog/2017/md/section-41-data-read-api.md:591` | annex_a | pure | complete | 3 |  | `class_item_qualifier` |
| 1800-2017 | `cmos_switchtype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1777` | annex_a+snapshot | pure | complete | 2 |  | `cmos_switchtype` |
| 1800-2017 | `constraint_prototype_qualifier` | `docs/systemverilog/2017/md/section-41-data-read-api.md:677` | annex_a+snapshot | pure | complete | 2 |  | `constraint_prototype_qualifier` |
| 1800-2017 | `data_type_or_void` | `docs/systemverilog/2017/md/section-41-data-read-api.md:916` | annex_a+snapshot | mixed | complete | 1 |  | `data_type_or_void` `data_type_or_void_sv_only` |
| 1800-2017 | `default_clause` | `docs/systemverilog/2017/md/section-41-data-read-api.md:424` | annex_a+snapshot | pure | complete | 1 |  | `default_clause` |
| 1800-2017 | `default_nettype_value` | `grammars/systemverilog_2017_lrm_extracted.ebnf:271` | snapshot | mixed | complete | 10 | `none` | `default_nettype_value` |
| 1800-2017 | `dpi_function_import_property` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1115` | annex_a+snapshot | pure | complete | 2 |  | `dpi_function_import_property` |
| 1800-2017 | `dpi_task_import_property` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1116` | annex_a+snapshot | pure | complete | 1 |  | `dpi_task_import_property` |
| 1800-2017 | `edge_identifier` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2692` | annex_a+snapshot | pure | complete | 3 |  | `edge_identifier` |
| 1800-2017 | `enable_gatetype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1778` | annex_a+snapshot | pure | complete | 4 |  | `enable_gatetype` |
| 1800-2017 | `import_export` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1211` | annex_a+snapshot | pure | complete | 2 |  | `import_export` |
| 1800-2017 | `integer_atom_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:888` | annex_a+snapshot | pure | complete | 6 |  | `integer_atom_type` `integer_atom_type_sv_only` |
| 1800-2017 | `integer_vector_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:889` | annex_a+snapshot | pure | complete | 3 |  | `integer_vector_type` `integer_vector_type_sv_only` |
| 1800-2017 | `join_keyword` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2076` | annex_a+snapshot | pure | complete | 3 |  | `join_keyword` |
| 1800-2017 | `let_formal_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1685` | annex_a | mixed | complete | 1 |  | `let_formal_type` |
| 1800-2017 | `lifetime` | `docs/systemverilog/2017/md/section-41-data-read-api.md:835` | annex_a+snapshot | pure | complete | 2 |  | `lifetime` |
| 1800-2017 | `module_keyword` | `docs/systemverilog/2017/md/section-41-data-read-api.md:120` | annex_a+snapshot | pure | complete | 2 |  | `module_keyword` |
| 1800-2017 | `mos_switchtype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1779` | annex_a+snapshot | pure | complete | 4 |  | `mos_switchtype` |
| 1800-2017 | `n_input_gatetype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1780` | annex_a+snapshot | pure | complete | 6 |  | `n_input_gatetype` |
| 1800-2017 | `n_output_gatetype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1781` | annex_a+snapshot | pure | complete | 2 |  | `n_output_gatetype` |
| 1800-2017 | `net_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:899` | annex_a+snapshot | pure | complete | 12 |  | `net_type` |
| 1800-2017 | `non_integer_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:898` | annex_a+snapshot | pure | complete | 3 |  | `non_integer_type` `non_integer_type_sv_only` |
| 1800-2017 | `pass_en_switchtype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1782` | annex_a+snapshot | pure | complete | 4 |  | `pass_en_switchtype` |
| 1800-2017 | `pass_switchtype` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1783` | annex_a+snapshot | pure | complete | 2 |  | `pass_switchtype` |
| 1800-2017 | `port_direction` | `docs/systemverilog/2017/md/section-41-data-read-api.md:283` | annex_a+snapshot | pure | complete | 4 |  | `port_direction` `port_direction_sv_only` |
| 1800-2017 | `property_formal_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1308` | annex_a | mixed | complete | 1 |  | `property_formal_type` |
| 1800-2017 | `property_lvar_port_direction` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1307` | annex_a+snapshot | pure | complete | 1 |  | `property_lvar_port_direction` |
| 1800-2017 | `random_qualifier` | `docs/systemverilog/2017/md/section-41-data-read-api.md:604` | annex_a | pure | complete | 2 |  | `random_qualifier` |
| 1800-2017 | `scope_type` | `grammars/systemverilog_2017_lrm_extracted.ebnf:964` | snapshot | pure | unmatched | 1 |  | _(none)_ |
| 1800-2017 | `sequence_formal_type` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1397` | annex_a | mixed | complete | 2 |  | `sequence_formal_type` |
| 1800-2017 | `sequence_lvar_port_direction` | `docs/systemverilog/2017/md/section-41-data-read-api.md:1388` | annex_a+snapshot | pure | complete | 3 |  | `sequence_lvar_port_direction` |
| 1800-2017 | `signing` | `docs/systemverilog/2017/md/section-41-data-read-api.md:909` | annex_a+snapshot | pure | complete | 2 |  | `signing` |
| 1800-2017 | `strength0` | `docs/systemverilog/2017/md/section-41-data-read-api.md:935` | annex_a+snapshot | pure | complete | 4 |  | `strength0` |
| 1800-2017 | `strength1` | `docs/systemverilog/2017/md/section-41-data-read-api.md:936` | annex_a+snapshot | pure | complete | 4 |  | `strength1` |
| 1800-2017 | `timing_check_event_control` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2832` | annex_a | mixed | complete | 3 |  | `timing_check_event_control` |
| 1800-2017 | `unique_priority` | `docs/systemverilog/2017/md/section-41-data-read-api.md:2212` | annex_a+snapshot | pure | complete | 3 |  | `unique_priority` |
| 1800-2023 | `always_keyword` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2022` | annex_a+snapshot | pure | complete | 4 |  | `always_keyword` `always_keyword_sv_only` |
| 1800-2023 | `array_method_name` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2943` | annex_a+snapshot | mixed | complete | 4 |  | `array_method_name` |
| 1800-2023 | `assignment_pattern_key` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2294` | annex_a+snapshot | mixed | complete | 1 |  | `assignment_pattern_key` |
| 1800-2023 | `bins_keyword` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1592` | annex_a+snapshot | pure | complete | 3 |  | `bins_keyword` |
| 1800-2023 | `case_keyword` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2233` | annex_a+snapshot | pure | complete | 3 |  | `case_keyword` |
| 1800-2023 | `casting_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:816` | annex_a+snapshot | mixed | complete | 2 |  | `casting_type` |
| 1800-2023 | `checker_port_direction` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:477` | annex_a+snapshot | pure | complete | 2 |  | `checker_port_direction` |
| 1800-2023 | `class_constructor_arg` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:557` | annex_a+snapshot | mixed | complete | 1 |  | `class_constructor_arg` `class_constructor_arg_sv_2023` |
| 1800-2023 | `class_item_qualifier` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:574` | annex_a+snapshot | pure | complete | 3 |  | `class_item_qualifier` |
| 1800-2023 | `cmos_switchtype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1786` | annex_a+snapshot | pure | complete | 2 |  | `cmos_switchtype` |
| 1800-2023 | `constraint_prototype_qualifier` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:654` | annex_a+snapshot | pure | complete | 2 |  | `constraint_prototype_qualifier` |
| 1800-2023 | `data_type_or_void` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:927` | annex_a+snapshot | mixed | complete | 1 |  | `data_type_or_void` `data_type_or_void_sv_only` |
| 1800-2023 | `default_clause` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:385` | annex_a+snapshot | pure | complete | 1 |  | `default_clause` |
| 1800-2023 | `default_nettype_value` | `grammars/systemverilog_2023_lrm_extracted.ebnf:340` | snapshot | mixed | complete | 10 | `none` | `default_nettype_value` |
| 1800-2023 | `dpi_function_import_property` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1156` | annex_a+snapshot | pure | complete | 2 |  | `dpi_function_import_property` |
| 1800-2023 | `dpi_task_import_property` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1157` | annex_a+snapshot | pure | complete | 1 |  | `dpi_task_import_property` |
| 1800-2023 | `edge_identifier` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2609` | annex_a+snapshot | pure | complete | 3 |  | `edge_identifier` |
| 1800-2023 | `enable_gatetype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1787` | annex_a+snapshot | pure | complete | 4 |  | `enable_gatetype` |
| 1800-2023 | `import_export` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1255` | annex_a+snapshot | pure | complete | 2 |  | `import_export` |
| 1800-2023 | `integer_atom_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:885` | annex_a+snapshot | pure | complete | 6 |  | `integer_atom_type` `integer_atom_type_sv_only` |
| 1800-2023 | `integer_vector_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:886` | annex_a+snapshot | pure | complete | 3 |  | `integer_vector_type` `integer_vector_type_sv_only` |
| 1800-2023 | `join_keyword` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2078` | annex_a+snapshot | pure | complete | 3 |  | `join_keyword` |
| 1800-2023 | `let_formal_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1700` | annex_a | mixed | complete | 1 |  | `let_formal_type` |
| 1800-2023 | `lifetime` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:810` | annex_a+snapshot | pure | complete | 2 |  | `lifetime` |
| 1800-2023 | `module_keyword` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:108` | annex_a+snapshot | pure | complete | 2 |  | `module_keyword` |
| 1800-2023 | `mos_switchtype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1788` | annex_a+snapshot | pure | complete | 4 |  | `mos_switchtype` |
| 1800-2023 | `n_input_gatetype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1789` | annex_a+snapshot | pure | complete | 6 |  | `n_input_gatetype` |
| 1800-2023 | `n_output_gatetype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1790` | annex_a+snapshot | pure | complete | 2 |  | `n_output_gatetype` |
| 1800-2023 | `net_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:888` | annex_a | pure | complete | 12 |  | `net_type` |
| 1800-2023 | `non_integer_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:887` | annex_a+snapshot | pure | complete | 3 |  | `non_integer_type` `non_integer_type_sv_only` |
| 1800-2023 | `pass_en_switchtype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1791` | annex_a+snapshot | pure | complete | 4 |  | `pass_en_switchtype` |
| 1800-2023 | `pass_switchtype` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1792` | annex_a+snapshot | pure | complete | 2 |  | `pass_switchtype` |
| 1800-2023 | `port_direction` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:256` | annex_a+snapshot | pure | complete | 4 |  | `port_direction` `port_direction_sv_only` |
| 1800-2023 | `property_formal_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1347` | annex_a | mixed | complete | 1 |  | `property_formal_type` |
| 1800-2023 | `property_lvar_port_direction` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1346` | annex_a+snapshot | pure | complete | 1 |  | `property_lvar_port_direction` |
| 1800-2023 | `random_qualifier` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:581` | annex_a+snapshot | pure | complete | 2 |  | `random_qualifier` |
| 1800-2023 | `scope_type` | `grammars/systemverilog_2023_lrm_extracted.ebnf:1105` | snapshot | pure | unmatched | 1 |  | _(none)_ |
| 1800-2023 | `sequence_formal_type` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1420` | annex_a | mixed | complete | 2 |  | `sequence_formal_type` |
| 1800-2023 | `sequence_lvar_port_direction` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1419` | annex_a+snapshot | pure | complete | 3 |  | `sequence_lvar_port_direction` |
| 1800-2023 | `signing` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:907` | annex_a+snapshot | pure | complete | 2 |  | `signing` |
| 1800-2023 | `strength0` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:970` | annex_a+snapshot | pure | complete | 4 |  | `strength0` |
| 1800-2023 | `strength1` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:971` | annex_a+snapshot | pure | complete | 4 |  | `strength1` |
| 1800-2023 | `timing_check_event_control` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2809` | annex_a | mixed | complete | 3 |  | `timing_check_event_control` |
| 1800-2023 | `unique_priority` | `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2210` | annex_a+snapshot | pure | complete | 3 |  | `unique_priority` |
| 1364-2005 | `cmos_switchtype` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:475` | annex_a+snapshot | pure | complete | 2 |  | `cmos_switchtype` |
| 1364-2005 | `default_clause` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:142` | annex_a+snapshot | pure | complete | 1 |  | `default_clause` |
| 1364-2005 | `default_nettype_value` | `grammars/verilog_2005_lrm_extracted.ebnf:215` | snapshot | mixed | complete | 10 | `none` | `default_nettype_value` |
| 1364-2005 | `edge_identifier` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:883` | annex_a+snapshot | pure | complete | 2 |  | `edge_identifier` |
| 1364-2005 | `enable_gatetype` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:476` | annex_a+snapshot | pure | complete | 4 |  | `enable_gatetype` |
| 1364-2005 | `file_size` | `grammars/verilog_2005_lrm_extracted.ebnf:404` | snapshot | pure | unmatched | 1 |  | _(none)_ |
| 1364-2005 | `logic` | `grammars/verilog_2005_lrm_extracted.ebnf:710` | snapshot | pure | unmatched | 4 |  | _(none)_ |
| 1364-2005 | `module_keyword` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:52` | annex_a+snapshot | pure | complete | 2 |  | `module_keyword` |
| 1364-2005 | `mos_switchtype` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:477` | annex_a+snapshot | pure | complete | 4 |  | `mos_switchtype` |
| 1364-2005 | `n_input_gatetype` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:478` | annex_a+snapshot | pure | complete | 6 |  | `n_input_gatetype` |
| 1364-2005 | `n_output_gatetype` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:479` | annex_a+snapshot | pure | complete | 2 |  | `n_output_gatetype` |
| 1364-2005 | `net_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:217` | annex_a+snapshot | pure | complete | 11 |  | `net_type` |
| 1364-2005 | `output_variable_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:221` | annex_a+snapshot | pure | unmatched | 2 |  | _(none)_ |
| 1364-2005 | `parameter_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:161` | annex_a+snapshot | pure | unmatched | 4 |  | _(none)_ |
| 1364-2005 | `pass_en_switchtype` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:480` | annex_a+snapshot | pure | complete | 4 |  | `pass_en_switchtype` |
| 1364-2005 | `pass_switchtype` | `grammars/verilog_2005_lrm_extracted.ebnf:956` | snapshot | pure | complete | 2 |  | `pass_switchtype` |
| 1364-2005 | `scope_type` | `grammars/verilog_2005_lrm_extracted.ebnf:1100` | snapshot | pure | unmatched | 5 |  | _(none)_ |
| 1364-2005 | `strength0` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:248` | annex_a+snapshot | pure | complete | 4 |  | `strength0` |
| 1364-2005 | `strength1` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:249` | annex_a+snapshot | pure | complete | 4 |  | `strength1` |
| 1364-2005 | `task_port_type` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:379` | annex_a+snapshot | pure | unmatched | 4 |  | _(none)_ |
| 1364-2005 | `timing_check_event_control` | `docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:981` | annex_a+snapshot | mixed | complete | 2 |  | `timing_check_event_control` |
| 1364-2005 | `var_type` | `grammars/verilog_2005_lrm_extracted.ebnf:1445` | snapshot | pure | unmatched | 18 |  | _(none)_ |

## Honest bound

- **Text-level.** A `present` verdict means the shipped rule's closure REACHES the
  terminal under the profile; it does not prove an input using the keyword parses in
  every context the LRM allows. A `MISSING` verdict is a triage row for the probe.
- **Bare-member closure, strict on purpose.** Only an alternative that is exactly one
  reference (or one parenthesised group) contributes; `kw_x kw_y?` does not count as
  `x`. That errs toward reporting, which is the direction nothing else catches.
- **Name-matched.** A production the shipped grammar renamed or lifted is `unmatched`,
  never `MISSING`. The unmatched list IS the bound on what this census could not see.
- **Source provenance.** The LRM side is the Annex A markdown first and the tracked
  extraction snapshot second, and the two are held equal on every shared enumeration.
  A production that IEEE defines only in a clause body and that neither surface carries
  is the business of `stimuli/sv/lrm_annex_a_gap_census.py` (TOOLBOX 5.8).
- **One direction.** Under-acceptance only. Over-acceptance candidates are printed and
  not scored (see *Extra members*).
