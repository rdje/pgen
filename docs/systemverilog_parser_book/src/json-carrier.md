# The Json Carrier

This chapter is a flat reference table of every `systemverilog.ebnf` rule that carries a `-> ...` return annotation, and the JSON shape that annotation produces.

> **Note:** The systemverilog return-annotation campaign is **in early phase**. Most rules are currently un-annotated and produce the recursive-envelope shape described in [AST Envelope Structure](ast-envelope.md). This table grows as the campaign progresses.

## Currently annotated rules

| Rule | Annotation | JSON shape produced |
|---|---|---|
| `systemverilog_file` | `-> {type: "systemverilog_file", source_text: $2}` | `{"type": "systemverilog_file", "source_text": <source_text-shape>}` — root JSON object for any `sv_2017` / `sv_2023` parse. `source_text` field is a flat array of `source_text_item` shapes (since SV-Slice-2). |
| `systemverilog_parseable_file` | `-> {type: "systemverilog_parseable_file", items: $2}` | `{"type": "systemverilog_parseable_file", "items": <parseable_source_item*-shape>}` — alternative entry rule for the parseable-source profile. `items` field carries the array of parseable source items in their raw envelope shape. |
| `source_text` | `-> [$1**]` | `[<source_text_item-typed-shape>, ...]` — flat array of typed source-text items via flatten-spread. Each item is a `{kind: "<name>", body: <envelope>}` typed object (per SV-Slice-3). |
| `source_text_item` (8 branches) | per-branch `{kind: "<name>", body: $1}` (or `{kind: "semi"}` for branch 7) | Typed object with `kind` discriminator: `"description"`, `"local_parameter_declaration"`, `"parameter_declaration"`, `"package_import_declaration"`, `"timeunits_declaration"`, `"compiler_directive"`, `"comment_only_source_region"`, `"semi"`. The `body` field carries the matched sub-rule's raw envelope OR a typed sub-rule shape if that sub-rule is itself annotated. For `kind: "description"`, body is now itself typed (per SV-Slice-4). The `semi` branch carries no `body` since it's just a stray `;`. Trailing `semi` dropped in branches 1 and 2 (annotation references `$1` only). |
| `description` (8 branches) | per-branch `{kind: "<name>", body: $1}` for single-element branches; `{kind: "<name>", attributes: $1, body: $2}` for multi-element branches with `attribute_instance*` prefix | Typed object with `kind` discriminator: `"module_declaration"`, `"udp_declaration"`, `"interface_declaration"`, `"program_declaration"`, `"package_declaration"`, `"package_item"`, `"bind_directive"`, `"config_declaration"`. The `attributes` field (only on `package_item` / `bind_directive` branches) carries the leading `attribute_instance*` iteration. The `body` field carries the matched sub-rule's raw envelope (per-rule typing of `module_declaration`, etc. is a follow-up slice). |
| `compiler_directive` | `-> $2` (transparent passthrough of regex capture) | Clean JSON string carrying the matched directive text (backtick + directive name + arguments, e.g. `"`define FOO bar"`). Drops the leading `trivia` slot. When `source_text_item.kind == "compiler_directive"`, the body is now a directly-usable string. |
| `attribute_instance` | `-> {first: $2, rest: $3}` | Typed object `{first: <attr_spec shape>, rest: <( comma attr_spec )* iteration>}`. Drops the `attr_open` (`(*`) and `attr_close` (`*)`) delimiters. The `first` field carries the leading attr_spec; `rest` carries the trailing reps as a Quantified iteration where each entry is `[comma, attr_spec]`. Mixed-array spread `[$2, $3**]` is currently blocked by an annotation-language limitation; the cleaner flat-array form is deferred until that's resolved. |
| `module_declaration_sv_2017` (5 branches) | per-branch typed shapes, see contract section "Release 1.0.6 / Contract 1.0.6 Highlights" for the full annotation source | Typed object with `kind` discriminator: `"ansi"` / `"nonansi"` / `"wildcard"` / `"extern_nonansi"` / `"extern_ansi"`. Single-form branches expose `header / timeunits / items / end_label`. The wildcard branch additionally exposes `attributes / keyword / lifetime / name`. Extern branches expose only `header`. |
| `module_declaration_sv_2023` (5 branches) | per-branch typed shapes (same kind set as sv_2017) | Identical kind discriminator and field names as sv_2017. Wildcard branch's positional indices shift due to `dot star` (2 tokens) vs `dot_star` (1 token); user-visible AST is identical to sv_2017. |
| `module_keyword` (2 branches) | `-> {kind: "module"}` / `-> {kind: "macromodule"}` | Typed object with `kind` discriminator. Drops the keyword token (redundant with `kind`). When referenced by parent rules (e.g. module_ansi_header.keyword), the typed shape propagates automatically. |
| `lifetime` (2 branches) | `-> {kind: "static"}` / `-> {kind: "automatic"}` | Typed object. When `(lifetime)?` is matched, consumers see `{kind: "static"}` / `{kind: "automatic"}`. When un-matched, `[]` (existing convention). |
| `module_ansi_header` | `-> {attributes, keyword, lifetime, name, imports, parameters, ports}` | 7 named fields. `keyword:` is itself typed (per `module_keyword`); `lifetime:` is itself typed when matched (per `lifetime`); `attributes`/`imports`/`parameters`/`ports` are quantified or optional (consumer handles `[]` for empty). `name:` carries raw `module_identifier` envelope. |
| `module_nonansi_header` | `-> {attributes, keyword, lifetime, name, imports, parameters, ports}` | Same field names as module_ansi_header. Only `ports:` source rule differs (`list_of_ports` vs `(list_of_port_declarations)?`); the typed shape is identical for consumers. |
| `simple_identifier` | `-> $2` | Clean JSON string — the matched identifier name (e.g. `"m"`, `"foo"`, `"my_signal"`). Drops the leading `trivia` slot. |
| `escaped_identifier` | `-> $2` | Clean JSON string — the matched escaped identifier (e.g. `"\\foo"`). Drops the leading `trivia` slot. |
| `non_keyword_identifier` | `-> $2` | Clean JSON string — passes the inner `identifier` through after the negative lookahead's empty `$1` slot. Identifier is itself typed (transparent Or of escaped/simple). |
| `simple_identifier_no_scope` | `-> $2` | Clean JSON string — variant of simple_identifier with a `(?!...)` negative lookahead in the regex (prevents trailing `::` consumption). Same trivia-drop as simple_identifier. |
| `interface_ansi_header` | `-> {attributes, lifetime, name, imports, parameters, ports}` | 6 named fields (no keyword field — interface only has one keyword). Same field names as module_ansi_header otherwise. `name:` is a clean identifier string (inherited from SV-Slice-8). |
| `interface_nonansi_header` | `-> {attributes, lifetime, name, imports, parameters, ports}` | Same field names as interface_ansi_header. Only `ports:` source rule differs. |
| `interface_declaration_sv_2017` (5 branches) | per-branch typed shapes (full source in contract Highlights) | Typed object with `kind` discriminator: `"ansi"` / `"nonansi"` / `"wildcard"` / `"extern_nonansi"` / `"extern_ansi"`. Same kind labels and field structure as module_declaration_sv_2017. Wildcard branch positions: $1 attributes, $3 name, $8 timeunits, $9 items, $11 end_label. |
| `interface_declaration_sv_2023` (5 branches) | per-branch typed shapes (same kind set as sv_2017) | Identical to sv_2017 except wildcard branch positions shift due to `dot star` (2 tokens) vs `dot_star` (1 token): $9 timeunits, $10 items, $12 end_label. |
| `class_declaration_sv_2017` | `-> {virtual, lifetime, name, parameters, extends, implements, items, end_label}` | Single-sequence typed object. `virtual` carries the `(kw_virtual)?` slot (typed kw object when matched, `[]` when un-matched). `name` is a clean string (inherited from SV-Slice-8). `extends` and `implements` carry the optional inheritance / interface-implementation clauses. |
| `class_declaration_sv_2023` | `-> {virtual, final_specifier, name, parameters, extends, implements, items, end_label}` | Same shape as sv_2017 but uses `final_specifier:` instead of `lifetime:` (LRM-2023 semantic change). Mutually-exclusive across profiles — consumers walking either profile dispatch on whichever field is present. |
| `package_declaration` | `-> {attributes, lifetime, name, timeunits, items, end_label}` | Single-sequence typed object. `name` is a clean package_identifier string. ⚠️ Open follow-up: bare-package input `package p; endpackage\n` parse rejected at top-level despite annotation registering correctly — investigation pending. |
| `program_declaration_sv_2017` (5 branches) | per-branch typed shapes (mirror of module/interface 5-form pattern) | Kind labels: `"nonansi"` / `"ansi"` / `"wildcard"` / `"extern_nonansi"` / `"extern_ansi"`. Note: program rule lists nonansi BEFORE ansi (different from module/interface order) — but kind labels still discriminate uniformly across constructs. Wildcard branch positions: $1 attributes, $3 name, $8 timeunits, $9 items, $11 end_label. |
| `program_declaration_sv_2023` (5 branches) | per-branch typed shapes (same kind set as sv_2017) | Wildcard branch positions shift to $9 timeunits, $10 items, $12 end_label (due to `dot star` vs `dot_star`). |
| `program_ansi_header` | `-> {attributes, lifetime, name, imports, parameters, ports}` | Same 6 named fields as `interface_ansi_header`. `name:` is a clean program_identifier string. |
| `program_nonansi_header` | `-> {attributes, lifetime, name, imports, parameters, ports}` | Same field names as program_ansi_header. Only `ports:` source rule differs (`list_of_ports` vs `(list_of_port_declarations)?`). |
| `udp_ansi_declaration` | `-> {attributes, name, ports}` | 3 named fields. Drops kw_primitive, parens, semi. `ports:` is the raw `udp_declaration_port_list` shape (sub-rule typing pending). |
| `udp_nonansi_declaration` | `-> {attributes, name, ports}` | Same field names as udp_ansi_declaration. Only `ports:` source rule differs (`udp_port_list` vs `udp_declaration_port_list`). |
| `udp_declaration_sv_2017` (5 branches) | per-branch typed shapes (full source in contract Highlights) | Kind labels: `"nonansi"` / `"ansi"` / `"extern_nonansi"` / `"extern_ansi"` / `"wildcard"`. **Special: `nonansi` branch uses `port_decls: {first, rest}` workaround** for the `udp_port_declaration udp_port_declaration*` mini-mixed-array. |
| `udp_declaration_sv_2023` (5 branches) | per-branch typed shapes (same kind set as sv_2017) | Identical to sv_2017 except wildcard branch positions shift due to `dot star` (2 tokens) vs `dot_star` (1 token): port_decls $9, body $10, end_label $12. |
| `bind_directive` (2 branches) | per-branch typed shapes | Kind labels: `"scoped"` (with target_scope, optional instances, instantiation) / `"single"` (with target_instance, instantiation). |
| `bind_instantiation` (4 branches) | per-branch `{kind, body}` | Kind labels: `"program"` / `"module"` / `"interface"` / `"checker"` — uniform shape over the 4 instantiation forms. |
| `package_item` (4 branches) | per-branch `{kind, body}` | Kind labels: `"declaration"` (package_or_generate_item_declaration) / `"anonymous_program"` / `"export"` (package_export_declaration) / `"timeunits"` (timeunits_declaration). |
| `bind_target_scope` (2 branches) | per-branch `{kind, name}` | Kind labels: `"module"` / `"interface"`. `name` is a clean identifier string. |
| `bind_target_instance` | `-> {name, bit_select}` | 2-element typed object. `name` is the hierarchical_identifier, `bit_select` is the constant_bit_select. |
| `bind_target_instance_list` | `-> {first, rest}` | Mini-mixed-array workaround for `bind_target_instance (comma bind_target_instance)*`. |
| `interface_class_declaration` | `-> {name, parameters, extends, items, end_label}` | Single-sequence shape mirror to class_declaration. |
| `config_declaration` | `-> {name, local_params, design, rules, end_label}` | Single-sequence shape with config-specific fields. |
| `list_of_ports` | `-> {first, rest}` | Drops `lparen`/`rparen`; mini-mixed-array workaround for `port (comma port)*`. |
| `list_of_port_declarations` | `-> $2` | Transparent passthrough of the optional inner content (which is itself a 3-element envelope when populated). Drops parens. |
| `udp_port_list` | `-> {output, inputs: {first, rest}}` | UDP-specific port list (uses port identifiers, not full declarations). `output` is a clean string (output_port_identifier); `inputs` is the {first, rest} array of input_port_identifier strings. |
| `udp_declaration_port_list` | `-> {output, inputs: {first, rest}}` | Parallel shape to udp_port_list but inner sub-rules are full declarations (udp_output_declaration / udp_input_declaration) instead of identifier strings. |
| `anonymous_program` | `-> {items}` | Drops kw_program/semi/kw_endprogram; surfaces only the items list. Reachable via package_item.kind = "anonymous_program". |
| `package_export_declaration` (2 branches) | per-branch typed | Kind labels: `"wildcard"` (`export *::*;`, drops content) / `"explicit"` (with items: {first, rest}). |
| `port` (2 branches) | per-branch typed | Kind labels: `"expression"` (positional port — `expr` may be `[]` for empty placeholder) / `"named"` (`.name(expr)` form — `name` is clean identifier string). |
| `port_direction` (4 branches) | per-branch `{kind}` | Kind labels: `"input"` / `"output"` / `"inout"` / `"ref"`. |
| `package_import_declaration` | `-> {items: {first, rest}}` | Drops kw_import/semi; surfaces import items as {first, rest}. |
| `package_import_item` (2 branches) | per-branch typed | Kind labels: `"explicit"` (`pkg::name` with `package` and `name` as clean identifier strings) / `"wildcard"` (`pkg::*` with only `package`). |
| `udp_body` (2 branches) | per-branch `{kind, body}` | Kind labels: `"combinational"` / `"sequential"`. |
| `udp_input_declaration` | `-> {attributes, identifiers}` | Drops kw_input. `identifiers` is the typed list_of_udp_port_identifiers `{first, rest}`. |
| `udp_output_declaration` (2 branches) | per-branch typed | Kind labels: `"wire"` (no `assign` clause) / `"reg"` (with optional default). |
| `combinational_body` | `-> {entries: {first, rest}}` | Drops kw_table/kw_endtable. Entries are the truth-table rows. |
| `sequential_body` | `-> {initial, entries: {first, rest}}` | Preserves optional initial statement. |
| `list_of_udp_port_identifiers` | `-> {first, rest}` | Mini-mixed-array workaround for `port_identifier (comma port_identifier)*`. |
| `combinational_entry` | `-> {inputs, output}` | Single truth-table row for combinational UDP. |
| `sequential_entry` | `-> {inputs, current_state, next_state}` | Single state-transition row for sequential UDP. |
| `udp_initial_statement` | `-> {name, init_val}` | Initial value assignment for sequential UDP. |
| `module_item` (2 branches) | per-branch `{kind, body}` | Kind labels: `"port_declaration"` / `"non_port_item"`. |
| `module_or_generate_item` (5 branches) | per-branch `{kind, attributes, body}` | Kind labels: `"parameter_override"` / `"gate_instantiation"` / `"udp_instantiation"` / `"module_instantiation"` / `"module_common_item"`. Each carries leading `attribute_instance*` as `attributes`. |
| `module_or_generate_item_declaration` (5 branches) | per-branch typed | Kind labels: `"package_or_generate"` / `"genvar"` / `"clocking"` / `"default_clocking"` (with `name`) / `"default_disable_iff"` (with `expr`). |
| `non_port_module_item` (8 branches) | per-branch typed | Kind labels: `"generate_region"` / `"module_or_generate"` / `"specify_block"` / `"specparam_declaration"` (with `attributes`) / `"program_declaration"` / `"module_declaration"` / `"interface_declaration"` / `"timeunits_declaration"`. |
| `continuous_assign` (2 branches) | per-branch typed | Kind labels: `"net"` (with `drive_strength`, `delay`, `assignments`) / `"variable"` (with `delay_control`, `assignments`). |
| `interface_item` (2 branches) | per-branch `{kind, body}` | Kind labels: `"port_declaration"` / `"non_port_item"`. Mirrors `module_item`. |
| `interface_or_generate_item` (2 branches) | per-branch `{kind, attributes, body}` | Kind labels: `"module_common_item"` / `"extern_tf_declaration"`. Each carries leading `attribute_instance*` as `attributes`. |
| `non_port_interface_item` (6 branches) | per-branch typed | Kind labels: `"generate_region"` / `"interface_or_generate"` / `"program_declaration"` / `"modport_declaration"` / `"interface_declaration"` / `"timeunits_declaration"`. |
| `program_item` (2 branches) | per-branch `{kind, body}` | Kind labels: `"port_declaration"` / `"non_port_item"`. Mirrors `module_item`. |
| `non_port_program_item` (7 branches) | per-branch typed | Kind labels: `"continuous_assign"` (with `attributes`) / `"module_or_generate_item_declaration"` (with `attributes`) / `"initial_construct"` (with `attributes`) / `"final_construct"` (with `attributes`) / `"concurrent_assertion_item"` (with `attributes`) / `"timeunits_declaration"` / `"program_generate_item"`. First 5 preserve leading `attribute_instance*` prefix; last 2 are bare. |
| `module_common_item_sv_2017` (13 branches) | per-branch `{kind, body}` | Kind labels: `"module_or_generate_item_declaration"` / `"interface_instantiation"` / `"program_instantiation"` / `"assertion_item"` / `"bind_directive"` / `"continuous_assign"` / `"net_alias"` / `"initial_construct"` / `"final_construct"` / `"always_construct"` / `"loop_generate_construct"` / `"conditional_generate_construct"` / `"elaboration_system_task"`. Reached from `module_or_generate_item.kind == "module_common_item"` and `interface_or_generate_item.kind == "module_common_item"`. |
| `module_common_item_sv_2023` (13 branches) | per-branch `{kind, body}` | Same kind set as sv_2017 except last branch is `"elaboration_severity_system_task"` (per LRM 2023 expanded system-task family). |
| `module_common_item` | un-annotated (transparent profile-router) | Passes through to the matched profile-typed sub-rule. Same pattern as `module_declaration` / `interface_declaration` wrapper. |
| `package_or_generate_item_declaration_sv_2017` (14 branches) | per-branch `{kind, body}` (or `{kind: "semi"}` for branch 13) | Kind labels: `"local_parameter_declaration"` / `"parameter_declaration"` / `"net_declaration"` / `"dpi_import_export"` / `"data_declaration"` / `"task_declaration"` / `"function_declaration"` / `"checker_declaration"` / `"extern_constraint_declaration"` / `"class_declaration"` / `"class_constructor_declaration"` / `"covergroup_declaration"` / `"assertion_item_declaration"` / `"semi"`. The `local_parameter_declaration` and `parameter_declaration` branches drop trailing `semi` via `body: $1`; `semi` carries no body. Reached from `module_or_generate_item_declaration.kind == "package_or_generate"`. |
| `package_or_generate_item_declaration_sv_2023` (15 branches) | per-branch `{kind, body}` (or `{kind: "semi"}`) | Same as sv_2017 plus `"interface_class_declaration"` between `class_declaration` and `class_constructor_declaration` (per LRM 2023). |
| `package_or_generate_item_declaration` | un-annotated (transparent profile-router) | Same pattern as `module_common_item` wrapper. |
| `generate_region` | `-> {items: $2}` | Typed object with single `items` field carrying the matched `generate_item*` array. Drops `kw_generate` and `kw_endgenerate` keywords. Reached from `non_port_module_item.kind == "generate_region"`. |
| `generate_item` (3 branches) | per-branch `{kind, body}` | Kind labels: `"module_or_generate_item"` / `"interface_or_generate_item"` / `"checker_or_generate_item"`. Discriminates which form of generate-item was matched (module vs interface vs checker context). |
| `generate_block` (3 branches) | per-branch typed | Kind labels: `"anonymous"` (`{label, items, end_label}` for `begin ... end` form), `"labeled"` (`{name, label, items, end_label}` for `name : begin ... end` form), `"generate_item"` (`{body}` for bare-generate_item passthrough — no `begin`/`end` wrapping). The `label` field captures the optional inner `( colon generate_block_identifier )?` after `kw_begin`; `end_label` captures the trailing optional after `kw_end`. |
| `loop_generate_construct` | `-> {init: $3, condition: $5, step: $7, block: $9}` | Single-sequence typed. Drops `kw_for`, `lparen`, `rparen`, and the two semicolons. Reached from `module_common_item.kind == "loop_generate_construct"`. |
| `conditional_generate_construct` (2 branches) | per-branch `{kind, body}` | Kind labels: `"if"` / `"case"`. Reached from `module_common_item.kind == "conditional_generate_construct"`. |
| `if_generate_construct` | `-> {condition: $3, then_block: $5, else_clause: $6}` | Single-sequence typed. `condition` is the matched constant_expression. `then_block` is the matched generate_block. `else_clause` is `[]` when no else is present, or `[<if_generate_else_clause shape>]` when one is matched. Drops `kw_if`, `lparen`, `rparen`. Reached from `conditional_generate_construct.kind == "if"`. |
| `if_generate_else_clause` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"elseif"` (body is itself an if_generate_construct — supports `else if` chains) / `"else_block"` (body is a generate_block — terminal else). Helper rule extracted from inline `( kw_else if_generate_construct \| kw_else generate_block )?` to dodge task #38. |
| `case_generate_construct` | `-> {expr: $3, items: {first: $5, rest: $6}}` | Single-sequence typed. `expr` is the case-discriminant constant_expression. `items` is a {first, rest} mini-mixed-array (first matched case_generate_item + raw Quantified iteration of repeats). Drops `kw_case`, `lparen`, `rparen`, `kw_endcase`. |
| `case_generate_item` (2 branches) | per-branch typed | Kind labels: `"expr_list"` (`{exprs: {first, rest}, block}` — N-way label-list form `expr1, expr2, ... : block`; mini-mixed-array on `exprs`) / `"default"` (`{block}` — drops `kw_default` and the optional `( colon )?` separator). |
| `assertion_item` (2 branches) | per-branch `{kind, body}` | Kind labels: `"concurrent"` / `"deferred_immediate"`. Reached from `module_common_item.kind == "assertion_item"`. |
| `assertion_item_declaration` (3 branches) | per-branch `{kind, body}` | Kind labels: `"property"` / `"sequence"` / `"let"`. Reached from `package_or_generate_item_declaration.kind == "assertion_item_declaration"`. |
| `concurrent_assertion_item` (2 branches) | per-branch typed | Kind labels: `"statement"` (`{label, body}` — `label` is the optional `( block_identifier colon )?` per LRM A.6.10; `[]` when no label) / `"checker_instantiation"` (`{body}`). |
| `genvar_initialization` | `-> {genvar_keyword: $1, name: $2, value: $4}` | Single-sequence typed. `genvar_keyword` is `[]` when re-using a pre-declared genvar, `[<kw_genvar token>]` when declare-and-init form. Drops `assign`. Reached from `loop_generate_construct.init`. |
| `genvar_iteration` (3 branches) | per-branch typed | Kind labels: `"assign"` (`{name, op, value}` — op is the typed `assignment_operator`), `"prefix_inc_dec"` (`{op, name}`), `"postfix_inc_dec"` (`{name, op}`). Reached from `loop_generate_construct.step`. |
| `assignment_operator` (13 branches) | per-branch `{kind}` (no body) | Kind labels: `"assign"` / `"plus_assign"` / `"minus_assign"` / `"star_assign"` / `"slash_assign"` / `"percent_assign"` / `"and_assign"` / `"or_assign"` / `"xor_assign"` / `"shift_left_assign"` / `"shift_right_assign"` / `"arithmetic_shift_left_assign"` / `"arithmetic_shift_right_assign"`. Each branch matches a single keyword token; the bare `{kind}` shape is sufficient for operator-by-name dispatch. |
| `inc_or_dec_operator` (2 branches) | per-branch `{kind}` (no body) | Kind labels: `"plus_plus"` / `"minus_minus"`. Same bare-`{kind}` pattern as `assignment_operator`. |
| `data_declaration_sv_2017` (4 branches) | per-branch typed | Kind labels: `"variable_decl"` (`{const_keyword, var_keyword, lifetime, data_type, assignments}` — most common form) / `"type"` (`{body}`) / `"package_import"` (`{body}`) / `"net_type"` (`{body}`). Reached from `package_or_generate_item_declaration.kind == "data_declaration"` (sv_2017 profile). |
| `data_declaration_sv_2023` (4 branches) | per-branch typed | Same first 3 kinds as sv_2017; 4th kind is `"nettype"` (LRM 2023 renamed `net_type` → `nettype`). Profile-agnostic walks should accept both `"net_type"` and `"nettype"`. |
| `function_declaration_sv_2017` | `-> {lifetime: $2, body: $3}` | Single-sequence typed. Drops `kw_function`. `lifetime` is `[]` or typed `lifetime` shape. `body` is the typed `function_body_declaration`. |
| `function_declaration_sv_2023` | `-> {dynamic_override: $2, lifetime: $3, body: $4}` | Same as sv_2017 plus LRM 2023's `( dynamic_override_specifiers )?` slot. `dynamic_override` is `[]` when absent. |
| `function_body_declaration` | `-> {return_type: $1, name: $2, items: $4, statements: $5, end_label: $7}` | Drops `semi`, `kw_endfunction`. `return_type` is the matched `function_data_type_or_implicit` (function may return void / scalar / vector / struct). `name` is a clean function_identifier string (per SV-Slice-8 identifier propagation). |
| `task_declaration_sv_2017` | `-> {lifetime: $2, body: $3}` | Parallel to `function_declaration_sv_2017`. |
| `task_declaration_sv_2023` | `-> {dynamic_override: $2, lifetime: $3, body: $4}` | Parallel to `function_declaration_sv_2023`. |
| `task_body_declaration` | `-> {name: $1, items: $3, statements: $4, end_label: $6}` | Same shape as `function_body_declaration` but no `return_type` (task is void by definition). |
| `net_declaration_sv_2017` (3 branches) | per-branch typed | Kind labels: `"wire"` (`{net_type, strength, vector_scalar, data_type, delay, assignments}` — `strength` and `vector_scalar` are typed via helper rules `net_strength` / `net_vector_scalar`) / `"alias"` (`{net_type_id, delay_control, assignments}`) / `"interconnect"` (`{data_type, delay, name, dims, second}`). |
| `net_declaration_sv_2023` (3 branches) | per-branch typed | Same 3 kinds as sv_2017; alias branch uses field `nettype_id` instead of `net_type_id` (LRM 2023 nettype-vs-net_type rule rename). |
| `net_strength` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"drive"` (body is the matched drive_strength shape) / `"charge"` (body is the matched charge_strength shape). Helper rule extracted from inline `( drive_strength \| charge_strength )?` to dodge task #38. |
| `net_vector_scalar` (NEW; 2 branches) | per-branch `{kind}` (no body) | Kind labels: `"vectored"` / `"scalared"`. Helper rule extracted from inline `( kw_vectored \| kw_scalared )?` to dodge task #38. Bare `{kind}` shape — each branch matches a single keyword token. |
| `class_item_sv_2017` (8 branches) | per-branch typed | Kind labels: `"property"` / `"method"` / `"constraint"` / `"class"` / `"covergroup"` (first 5 with `attribute_instance*` prefix as `attributes`) / `"local_parameter"` / `"parameter"` (drop trailing `semi` via `body: $1`) / `"semi"` (bare, no body). Reached from `class_declaration_sv_2017.items[]`. |
| `class_item_sv_2023` (9 branches) | per-branch typed | Same 8 kinds as sv_2017 plus `"interface_class"` (with `attribute_instance*` prefix) — LRM 2023 allows nested interface-class declarations. |
| `class_item_qualifier` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"static"` / `"protected"` / `"local"`. Same bare-`{kind}` pattern as `assignment_operator` / `inc_or_dec_operator`. |
| `class_constraint` (2 branches) | per-branch `{kind, body}` | Kind labels: `"prototype"` / `"declaration"`. Reached from `class_item.kind == "constraint"`. |
| `class_property` (2 branches) | per-branch typed | Kind labels: `"decl"` (`{qualifiers, body}` — standard form `property_qualifier* data_declaration`; body is the typed data_declaration shape from SV-Slice-25) / `"const"` (`{qualifiers, data_type, name, init}` — kw_const-prefixed form; `qualifiers` is the typed `class_item_qualifier*` slot, `init` is `[]` when no initializer or `[<assign, expr>]` when present). |
| `class_method` (6 branches) | per-branch typed | Kind labels: `"task"` / `"function"` (`{qualifiers, body}` — body is typed task_declaration / function_declaration from SV-Slice-25) / `"pure_virtual"` (`{qualifiers, prototype}` — prototype-only, no body; `qualifiers` is typed `class_item_qualifier*` slot) / `"extern"` (`{qualifiers, prototype}` — prototype-only, kw_extern-prefixed) / `"constructor"` (`{qualifiers, body}` — class_constructor_declaration with full body) / `"extern_constructor"` (`{qualifiers, prototype}` — extern class new). |
| `method_qualifier` (2 branches) | per-branch typed | Kind labels: `"virtual"` (`{pure}` — `pure` is `[]` for bare virtual, `[<kw_pure>]` for `pure virtual`) / `"class_item_qualifier"` (`{body}` — body is typed class_item_qualifier shape: static/protected/local). Reached from `class_method.qualifiers[]`. |
| `property_qualifier` (2 branches) | per-branch `{kind, body}` | Kind labels: `"random"` (body is typed random_qualifier: rand/randc) / `"class_item_qualifier"` (body is typed class_item_qualifier: static/protected/local). Reached from `class_property.qualifiers[]`. |
| `random_qualifier` (2 branches) | per-branch `{kind}` (bare) | Kind labels: `"rand"` / `"randc"`. Same bare-`{kind}` pattern as `class_item_qualifier`. |
| `concurrent_assertion_statement` (5 branches) | per-branch `{kind, body}` | Kind labels: `"assert_property"` / `"assume_property"` / `"cover_property"` / `"cover_sequence"` / `"restrict_property"`. Reached from `concurrent_assertion_item.kind == "statement"` body. |
| `assert_property_statement` | `-> {spec, action}` | Drops `kw_assert`, `kw_property`, parens. `spec` is the property_spec; `action` is the action_block (assert can branch on pass/fail). |
| `assume_property_statement` | `-> {spec, action}` | Parallel to assert. |
| `cover_property_statement` | `-> {spec, statement}` | Cover uses `statement_or_null` instead of action_block (no pass/fail branching). |
| `cover_sequence_statement` | `-> {clocking, disable_iff, sequence, statement}` | Different from cover_property: covers a sequence_expr with optional clocking_event prefix and optional `disable iff (...)` clause. |
| `restrict_property_statement` | `-> {spec}` | No action — restrict prunes formal traces but doesn't branch on outcome. |
| `expect_property_statement` | `-> {spec, action}` | Same shape as assert. |
| `constraint_block` | `-> {items: $2}` | Drops braces. `items` is the constraint_block_item* iteration. |
| `constraint_block_item` (2 branches) | per-branch typed | Kind labels: `"solve_before"` (`{before, after}` — drops kw_solve / kw_before / semi) / `"expression"` (`{body}` — wraps a constraint_expression). |
| `constraint_declaration_sv_2017` | `-> {static_keyword, name, block}` | `static_keyword` is `[]` or `[<kw_static>]`. |
| `constraint_declaration_sv_2023` | `-> {static_keyword, dynamic_override, name, block}` | Adds `dynamic_override` slot per LRM 2023, parallel to function/task. |
| `constraint_expression` (6 branches) | per-branch typed | Kind labels: `"expression"` (`{soft, expr}` — `soft` is `[]` or `[<kw_soft>]`) / `"uniqueness"` (`{body}`) / `"implies"` (`{condition, body}` — `expression implies constraint_set`) / `"if"` (`{condition, then_body, else_clause}` — `else_clause` is `[]` or `[<kw_else, set>]`) / `"foreach"` (`{array, loop_vars, body}`) / `"disable_soft"` (`{target}`). |
| `constraint_prototype_sv_2017` | `-> {qualifier, static_keyword, name}` | `qualifier` is the optional `( constraint_prototype_qualifier )?` (typed). |
| `constraint_prototype_sv_2023` | `-> {qualifier, static_keyword, dynamic_override, name}` | Same as sv_2017 plus dynamic_override. |
| `constraint_prototype_qualifier` (2 branches) | per-branch `{kind}` (bare) | Kind labels: `"extern"` / `"pure"`. |
| `constraint_set` (2 branches) | per-branch typed | Kind labels: `"single"` (`{body}` — wraps a single constraint_expression) / `"block"` (`{exprs}` — brace-delimited list of constraint_expressions). |
| `deferred_immediate_assertion_item` | `-> {label, body}` | `label` is the optional `( block_identifier colon )?` prefix per LRM A.6.10 (parallel to concurrent_assertion_item.label from SV-Slice-24). |
| `deferred_immediate_assertion_statement` (3 branches) | per-branch `{kind, body}` | Kind labels: `"assert"` / `"assume"` / `"cover"`. |
| `deferred_immediate_assert_statement` (2 branches) | per-branch typed | Kind labels: `"zero_delay"` (`{expression, action}` — `assert #0 (expr) action_block`; LRM 1800-2017 §16.3.1 Re-NBA evaluation) / `"final"` (`{expression, action}` — `assert final (expr) action_block`; end-of-simulation evaluation). |
| `deferred_immediate_assume_statement` (2 branches) | per-branch typed | Same 2 kinds and shape as deferred_immediate_assert_statement (with `kw_assume` instead of `kw_assert`). |
| `deferred_immediate_cover_statement` (2 branches) | per-branch typed | Kind labels: `"zero_delay"` / `"final"` (parallel to assert/assume). Uses `statement` (statement_or_null) instead of `action` since cover has no pass/fail branching. |
| `action_block` (2 branches) | per-branch typed | Kind labels: `"always"` (`{body}` — runs unconditionally; LRM A.6.3 single statement_or_null form) / `"with_else"` (`{pass, fail}` — `[statement] else statement_or_null` form; `pass` is `[]` if pass-statement omitted, like `assert (x) else $error(...)`). |
| `statement` | `-> {label: $1, attributes: $2, body: $3}` | `label` is optional `( block_identifier colon !colon )?` — the `!colon` lookahead distinguishes block label from `::` package-scope-resolution. `body` is a typed `statement_item`. |
| `statement_or_null` (2 branches) | per-branch typed | Kind labels: `"statement"` (`{body}`) / `"null"` (`{attributes}` — bare `;` with optional preceding `attribute_instance*`). |
| `function_statement_or_null` (2 branches) | per-branch typed | Same shape as statement_or_null but for function bodies. |
| `tf_item_declaration` (2 branches) | per-branch `{kind, body}` | Kind labels: `"block_item"` / `"tf_port"`. Reached from `function_body_declaration.items[]` and `task_body_declaration.items[]`. |
| `statement_item_sv_2017` (20 branches) | per-branch `{kind, body}` | Kind labels: `"blocking_assignment"` / `"nonblocking_assignment"` / `"procedural_continuous_assignment"` (each drops trailing `semi`) / `"case"` / `"conditional"` / `"inc_or_dec_expression"` (drops `semi`; sv_2017-only) / `"subroutine_call"` / `"disable"` / `"event_trigger"` / `"loop"` / `"jump"` / `"par_block"` / `"procedural_timing_control"` / `"seq_block"` / `"wait"` / `"procedural_assertion"` / `"clocking_drive"` (drops `semi`) / `"randsequence"` / `"randcase"` / `"expect_property"`. Reached from `statement.body`. |
| `statement_item_sv_2023` (19 branches) | per-branch `{kind, body}` | Same 19 kinds as sv_2017 except `"inc_or_dec_expression"` is removed — LRM 2023 subsumes the semantics into `blocking_assignment` with `++`/`--` operators. |
| `block_item_declaration` (4 branches) | per-branch `{kind, attributes, body}` | Kind labels: `"block_data"` / `"local_parameter"` / `"parameter"` / `"let"`. Each preserves leading `attribute_instance*` prefix. Reached from `tf_item_declaration.kind == "block_item"`. |
| `disable_statement` (3 branches) | per-branch typed | Kind labels: `"task"` (`{target}`) / `"block"` (`{target}`) / `"fork"` (bare). Reached from `statement_item.kind == "disable"`. |
| `jump_statement` (3 branches) | per-branch typed | Kind labels: `"return"` (`{value}` — value is `[]` for bare `return;` or `[<expr>]` for `return expr;`) / `"break"` (bare) / `"continue"` (bare). |
| `wait_statement` (3 branches) | per-branch typed | Kind labels: `"wait"` (`{condition, body}`) / `"wait_fork"` (bare) / `"wait_order"` (`{events: {first, rest}, action}` — wait order(e1, e2, ..., eN) action_block; mini-mixed-array on events). |
| `event_trigger_sv_2017` (2 branches) | per-branch typed | Kind labels: `"non_blocking"` (`{name}` — `-> name;`) / `"blocking"` (`{control, name}` — `->> [delay] name;`). |
| `event_trigger_sv_2023` (2 branches) | per-branch typed | Same 2 kinds as sv_2017 plus a `select` field per branch (LRM 2023 nonrange_select extension). |
| `procedural_timing_control_statement` | `-> {control: $1, body: $2}` | Standard timing-controlled statement form: `<control> statement_or_null`. Reached from `statement_item.kind == "procedural_timing_control"`. |
| `procedural_timing_control` (3 branches) | per-branch `{kind, body}` | Kind labels: `"delay"` (#N delay) / `"event"` (@(...) event) / `"cycle"` (##N cycle delay). |
| `subroutine_call` (5 branches) | per-branch typed | Kind labels: `"class_scoped_tf"` (`{body}`) / `"tf"` (`{body}`) / `"system_tf"` (`{body}` — $display etc.) / `"method"` (`{body}`) / `"randomize"` (`{std_scope, body}` — std_scope is `[]` for plain randomize, `[<kw_std, ::>]` for std::randomize). |
| `subroutine_call_statement` (2 branches) | per-branch typed | Kind labels: `"call"` (`{body}` — wraps subroutine_call with trailing `;`) / `"void_cast"` (`{body}` — `void'(func_call);` discards return value). |
| `seq_block` | `-> {label, declarations, statements, end_label}` | Drops `kw_begin` / `kw_end`. `label` is optional `( colon block_identifier )?` (e.g., `begin : my_block`). |
| `par_block` | `-> {label, declarations, statements, join, end_label}` | Same as seq_block plus `join` field carrying the typed `join_keyword` (discriminates `join` / `join_any` / `join_none` per SV-Slice-7). |
| `case_statement` | `-> {unique_priority, keyword, expr, items: {first, rest}}` | Drops `kw_endcase` and parens. `keyword` is the typed `case_keyword` (case/casez/casex). `items` is mini-mixed-array with first matched case_item + rest iteration. `unique_priority` raw envelope (still — see DEFERRED in slice 34 contract). |
| `case_keyword` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"case"` / `"casez"` / `"casex"`. |
| `case_item` (2 branches) | per-branch typed | Kind labels: `"expr_list"` (`{exprs: {first, rest}, body}` — N-way label-list `expr1, expr2, ... : stmt;`) / `"default"` (`{body}` — drops `kw_default` and optional colon). Same shape pattern as case_generate_item from SV-Slice-23. |
| `case_pattern_item` (2 branches) | per-branch typed | Kind labels: `"pattern"` (`{pattern, condition, body}` — `condition` is the optional `&&& expression` guard per LRM A.6.7.1) / `"default"` (`{body}`). |
| `case_inside_item_sv_2017` (2 branches) | per-branch typed | Kind labels: `"range_list"` (`{ranges, body}` — ranges is open_range_list) / `"default"` (`{body}`). |
| `case_inside_item_sv_2023` (2 branches) | per-branch typed | Same kind labels as sv_2017; ranges field uses LRM 2023 `range_list` (simplified naming). |
| `loop_statement` (6 branches) | per-branch typed | Kind labels: `"forever"` (`{body}`) / `"repeat"` (`{count, body}`) / `"while"` (`{condition, body}`) / `"for"` (`{init, condition, step, body}` — each of init/condition/step is `[]` when omitted) / `"do_while"` (`{body, condition}`) / `"foreach"` (`{array, loop_vars, body}` — body is a typed `statement`, not statement_or_null since bare `;` not allowed). |
| `conditional_statement` | `-> {unique_priority, condition, then_body, else_body}` | Drops `kw_if` / `lparen` / `rparen` / `kw_else`. `condition` is typed `cond_predicate` (raw envelope still). `then_body` is a typed `statement_or_null`. `else_body` is a typed `conditional_else_branch` (helper rule). The rule preserves a `&kw_else` positive lookahead before consuming `kw_else` — PEG idiom from the source grammar. |
| `conditional_else_branch` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"elseif"` (recursive — `body` is itself a typed `conditional_statement`; supports `else if (...) ...` chains) / `"else"` (terminal — `body` is a typed `statement_or_null`). Helper rule extracted from inline `( conditional_statement \| statement_or_null )` to dodge task #38. |
| `nonblocking_assignment` | `-> {lvalue, control, value}` | Drops `<=`. `control` is `[]` for `a <= b;`, `[<delay_or_event_control shape>]` for `a <= #1 b;`. |
| `procedural_continuous_assignment` (6 branches) | per-branch typed | Kind labels: `"assign"` (`{body}` — kw_assign + variable_assignment) / `"deassign"` (`{target}`) / `"force_variable"` (`{body}`) / `"force_net"` (`{body}`) / `"release_variable"` (`{target}`) / `"release_net"` (`{target}`). The split between force/release variants reflects the grammar's separate variable_assignment vs net_assignment / variable_lvalue vs net_lvalue branches. |
| `clocking_drive` | `-> {lvalue, cycle_delay, value}` | Drops `<=`. `cycle_delay` is the optional `( cycle_delay )?` slot (clocking-block specific delay). |
| `randcase_statement` | `-> {items: {first: $2, rest: $3}}` | Drops `kw_randcase` / `kw_endcase`. Mini-mixed-array on items. |
| `randcase_item` | `-> {weight: $1, body: $3}` | Drops colon. `weight` is the relative selection weight expression. |
| `procedural_assertion_statement` (3 branches) | per-branch `{kind, body}` | Kind labels: `"concurrent"` (typed in SV-Slice-29) / `"immediate"` (typed in this slice — bridges to immediate_assertion_statement) / `"checker_instantiation"` (raw envelope still). |
| `immediate_assertion_statement` (2 branches) | per-branch `{kind, body}` | Kind labels: `"simple"` (raw envelope still — `simple_immediate_assertion_statement` to be typed in a future slice) / `"deferred"` (typed in SV-Slice-30 — `deferred_immediate_assertion_statement`). |
| `variable_assignment` | `-> {lvalue, value}` | Drops `assign`. Reached from `procedural_continuous_assignment.kind == "assign"` and from `force_variable`. |
| `blocking_assignment_sv_2017` (4 branches) | per-branch typed | Kind labels: `"delay_assign"` (`{lvalue, delay, value}` — `lvalue = #N expr;`) / `"dynamic_array_new"` (`{lvalue, value}` — `lvalue = new[size];`) / `"class_new"` (`{scope, name, select, value}` — `[scope.]name[select] = new(args);`; `scope` uses NEW helper rule) / `"operator"` (`{body}` — wraps operator_assignment for `+=` / `*=` / etc.). |
| `blocking_assignment_sv_2023` (5 branches) | per-branch typed | Same 4 kinds as sv_2017 plus `"inc_or_dec"` (`{body}` — LRM 2023 folds `++` / `--` into blocking_assignment; in sv_2017 these were a separate `inc_or_dec_expression semi` statement_item branch). |
| `class_or_package_scope` (NEW; 3 branches) | per-branch typed | Kind labels: `"instance"` (`{handle}` — implicit_class_handle dot prefix, typically `this.` / `super.`) / `"class_scope"` (`{body}` — `ClassName::`) / `"package_scope"` (`{body}` — `pkg::`). Helper rule extracted from inline `( implicit_class_handle dot \| class_scope \| package_scope )?` to dodge task #38. 4th use of the helper-rule extraction pattern. |
| `randsequence_statement_sv_2017` | `-> {start, productions: {first, rest}}` | Drops `kw_randsequence`, parens, `kw_endsequence`. `start` is `[]` for `randsequence () ...`, `[<production_identifier>]` for `randsequence (top) ...`. `productions` is mini-mixed-array (first + rest). |
| `randsequence_statement_sv_2023` | `-> {start, productions: {first, rest}}` | Same shape as sv_2017. References `rs_production_identifier` / `rs_production` per LRM 2023 namespacing (the typed shape is identical for consumers). |
| `production_sv_2017` | `-> {return_type, name, ports, rules: {first, rest}}` | Drops `colon` and trailing `semi`. `return_type` is the optional `data_type_or_void` prefix. `ports` is the optional `( lparen tf_port_list rparen )?` slot. `rules` is mini-mixed-array — each entry in `rest` is a `[bitwise_or_token, rs_rule]` pair (alternative rules separated by `|`). |
| `production_item_sv_2017` | `-> {name, args}` | `args` is the optional `( lparen list_of_arguments rparen )?` slot — `[]` for plain reference, `[<arg list>]` when invoking with arguments. |
| `rs_case` | `-> {expr, items: {first, rest}}` | Drops `kw_case` / `kw_endcase` / parens. |
| `rs_case_item_sv_2017` (2 branches) | per-branch typed | Kind labels: `"expr_list"` (`{exprs: {first, rest}, body}`) / `"default"` (`{body}`). |
| `rs_case_item_sv_2023` (2 branches) | per-branch typed | Same kinds as sv_2017; uses rs_production_item per LRM 2023 namespacing. |
| `rs_code_block` | `-> {body}` | Exposes the Quantified iteration of `( data_declaration* statement_or_null* )*` directly — each entry is `[data_declaration*-array, statement_or_null*-array]`. |
| `rs_if_else_sv_2017` | `-> {condition, then_body, else_body}` | `else_body` is `[]` for if-without-else, `[<production_item>]` when present. |
| `rs_if_else_sv_2023` | `-> {condition, then_body, else_body}` | Parallel; uses rs_production_item. |
| `rs_prod_sv_2017` (5 branches) | per-branch `{kind, body}` | Kind labels: `"production_item"` / `"code_block"` / `"if_else"` / `"repeat"` / `"case"`. |
| `rs_prod_sv_2023` (5 branches) | per-branch `{kind, body}` | Same 5 kinds as sv_2017; first branch is rs_production_item. |
| `rs_production_sv_2023` | `-> {return_type, name, ports, rules: {first, rest}}` | Parallel to production_sv_2017 from SV-Slice-38. |
| `rs_production_item_sv_2023` | `-> {name, args}` | Parallel to production_item_sv_2017 from SV-Slice-38. |
| `rs_production_list_sv_2017` (2 branches) | per-branch typed | Kind labels: `"productions"` (`{items: {first, rest}}` — simple rs_prod sequence) / `"rand_join"` (`{join_count, items: {first, second, rest}}` — `rand join [(expr)] prod1 prod2 [prod3...]`; per LRM A.6.13 at least 2 production_items required). |
| `rs_production_list_sv_2023` (2 branches) | per-branch typed | Same kinds; rand_join uses rs_production_item. |
| `rs_repeat_sv_2017` | `-> {count, body}` | Drops kw_repeat / parens. |
| `rs_repeat_sv_2023` | `-> {count, body}` | Parallel; uses rs_production_item. |
| `rs_rule_sv_2017` | `-> {productions, weight}` | `weight` is the optional `( colon assign weight_specification ( rs_code_block )? )?` slot — `[]` when production has no explicit weight. |
| `rs_rule_sv_2023` | `-> {productions, weight}` | Parallel; uses rs_weight_specification per LRM 2023. |
| `rs_weight_specification_sv_2023` (3 branches) | per-branch `{kind, body}` | Kind labels: `"number"` (integral_number) / `"identifier"` (ps_identifier) / `"expression"` (parenthesized expression). |
| `simple_immediate_assertion_statement` (3 branches) | per-branch `{kind, body}` | Kind labels: `"assert"` / `"assume"` / `"cover"`. Reached from `immediate_assertion_statement.kind == "simple"`. |
| `simple_immediate_assert_statement` | `-> {condition, action}` | Drops `kw_assert` / parens. `action` is a typed `action_block`. |
| `simple_immediate_assume_statement` | `-> {condition, action}` | Parallel to assert. |
| `simple_immediate_cover_statement` | `-> {condition, statement}` | Uses `statement_or_null` instead of `action_block` (cover has no pass/fail branching). |
| `inc_or_dec_expression` (2 branches) | per-branch typed | Kind labels: `"prefix"` (`{op, attributes, lvalue}` — `++a` / `--a`) / `"postfix"` (`{lvalue, attributes, op}` — `a++` / `a--`). The `attributes` slot carries inline `attribute_instance*` (LRM allows attributes between operator and operand). |
| `weight_specification_sv_2017` (3 branches) | per-branch `{kind, body}` | Kind labels: `"number"` / `"identifier"` / `"expression"`. Parallel to `rs_weight_specification_sv_2023`. |
| `data_type` (15 branches) | per-branch typed | Kind labels: `"integer_vector"` (`{base, signing, dims}` — bit/logic/reg with optional signing + packed dims) / `"integer_atom"` (`{base, signing}` — byte/shortint/int/longint/integer/time) / `"non_integer"` (`{base}` — shortreal/real/realtime) / `"struct_union"` (`{header, packed_signing, members: {first, rest}, dims}`) / `"enum"` (`{base_type, names: {first, rest}, dims}`) / `"string"` / `"chandle"` / `"event"` (bare; no body) / `"virtual_interface"` (`{interface_keyword, name, params, modport}`) / `"scoped_data_type"` / `"known_unscoped_data_type"` / `"class_type"` / `"provisional_class_type"` / `"covergroup"` / `"type_reference"` (each `{body}`). |
| `data_type_or_implicit` (2 branches) | per-branch `{kind, body}` | Kind labels: `"data_type"` / `"implicit"`. |
| `data_type_or_void` (2 branches) | per-branch typed | Kind labels: `"data_type"` (`{body}`) / `"void"` (bare). |
| `data_type_or_incomplete_class_scoped_type_sv_2023` (2 branches) | per-branch `{kind, body}` | Kind labels: `"data_type"` / `"incomplete_class_scoped"`. |
| `implicit_data_type` | `-> {signing, dims}` | `signing` is `[]` or typed signing slot. `dims` is the packed_dimension* iteration. |
| `integer_atom_type` (6 branches) | per-branch `{kind}` (bare) | Kind labels: `"byte"` / `"shortint"` / `"int"` / `"longint"` / `"integer"` / `"time"`. |
| `integer_vector_type` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"bit"` / `"logic"` / `"reg"`. |
| `non_integer_type` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"shortreal"` / `"real"` / `"realtime"`. |
| `integer_type` (2 branches) | per-branch `{kind, body}` | Kind labels: `"vector"` / `"atom"`. |
| `signing` (2 branches) | per-branch `{kind}` (bare) | Kind labels: `"signed"` / `"unsigned"`. |
| `struct_union_sv_2017` (2 branches) | per-branch typed | Kind labels: `"struct"` (bare) / `"union"` (`{tagged}` — `tagged` is `[]` or `[<kw_tagged>]`). |
| `struct_union_sv_2023` (2 branches) | per-branch typed | Kind labels: `"struct"` / `"union"` (`{modifier}` — modifier is the optional typed `union_modifier` shape). |
| `union_modifier` (NEW; 2 branches) | per-branch `{kind}` (bare) | Kind labels: `"soft"` / `"tagged"`. Helper rule extracted from inline `( kw_soft \| kw_tagged )?` to dodge task #38. |
| `struct_union_member` | `-> {attributes, random_qualifier, data_type, decls}` | `random_qualifier` is `[]` or `[<rand/randc>]`. `data_type` is the typed `data_type_or_void`. `decls` is the variable_decl_assignments list. |
| `enum_base_type` (3 branches) | per-branch typed | Kind labels: `"atom"` (`{base, signing}`) / `"vector"` (`{base, signing, dim}`) / `"type_alias"` (`{name, dim}`). |
| `enum_name_declaration` | `-> {name, range, value}` | `range` is `[]` or `[<lbrack, n, [colon n], rbrack>]`. `value` is `[]` or `[<assign, expr>]`. |
| `type_reference_sv_2017` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"data_type"`. |
| `type_reference_sv_2023` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"data_type_or_incomplete_class"` (LRM 2023 widens the second variant). |
| `class_type` | `-> {head, params, suffix}` | `head` is the typed `class_type_head` (helper rule). `params` is `[]` or `[<parameter_value_assignment>]`. `suffix` is `*` of `[scope_resolution, class_identifier, [parameter_value_assignment]]` chains for nested class scopes. |
| `class_type_head` (NEW; 3 branches) | per-branch `{kind, body}` | Kind labels: `"scoped"` / `"class"` / `"interface_class"`. Helper rule extracted from leading 3-way parens-Or in class_type to dodge task #38. |
| `parameter_value_assignment_sv_2017` | `-> {params: $3}` | Drops `hash`, parens. `params` is `[]` for `#()`, `[<list_of_parameter_assignments shape>]` for non-empty form. |
| `parameter_value_assignment_sv_2023` | `-> {params: $3}` | Parallel; uses list_of_parameter_value_assignments per LRM 2023 naming. |
| `list_of_parameter_assignments_sv_2017` (2 branches) | per-branch `{kind, items: {first, rest}}` | Kind labels: `"ordered"` (positional `#(8, 16)`) / `"named"` (keyword `#(.N(8), .M(16))`). |
| `list_of_parameter_value_assignments_sv_2023` (2 branches) | per-branch `{kind, items: {first, rest}}` | Same kinds as sv_2017. |
| `named_parameter_assignment` | `-> {name: $2, value: $4}` | Drops `dot` / parens. `value` is `[]` for `.name()`, `[<param_expression>]` for `.name(expr)`. |
| `named_argument` | `-> {name: $2, value: $4}` | Same shape as named_parameter_assignment. |
| `list_of_arguments` (3 branches) | per-branch `{kind, body}` | Kind labels: `"ordered"` / `"named"` / `"mixed"` (positional + trailing named, e.g., `f(1, 2, .x(3))`). |
| `list_of_arguments_ordered` | `-> {first: $1, rest: $2}` | `first` is `[]` for empty arg, `[<expression>]` otherwise. |
| `list_of_arguments_named` | `-> {first: $1, rest: $2}` | All-named form. |
| `list_of_arguments_mixed` | `-> {head: $1, named: {first: $3, rest: $4}}` | `head` is the typed `list_of_arguments_mixed_head` (positional prefix). `named` is mini-mixed-array of trailing named arguments. |
| `list_of_arguments_mixed_head` (2 branches) | per-branch typed | Kind labels: `"single"` (`{body}` — single positional argument) / `"chain"` (`{expr, rest}` — recursive: optional expression followed by comma followed by recursive head). |
| `list_of_clocking_decl_assign` / `list_of_defparam_assignments` / `list_of_genvar_identifiers` / `list_of_net_assignments` / `list_of_net_decl_assignments` / `list_of_param_assignments` / `list_of_path_inputs` / `list_of_path_outputs` / `list_of_specparam_assignments` / `list_of_type_assignments` / `list_of_variable_assignments` / `list_of_variable_decl_assignments` (12 rules) | `-> {first, rest}` | Uniform mini-mixed-array. `first` is the leading required item; `rest` is the trailing iteration of `[comma, item]` pairs. |
| `list_of_interface_identifiers` / `list_of_port_identifiers` / `list_of_variable_identifiers` (3 rules) | `-> {first: {name, dims}, rest}` | Per-item `{name, dims}` (identifier + trailing unpacked/variable dimension list per LRM); `rest` is iteration of comma-prefixed items. |
| `list_of_tf_variable_identifiers` | `-> {first: {name, dims, init}, rest}` | Per-item adds `init: ( assign expression )?` slot. |
| `list_of_variable_port_identifiers` | `-> {first: {name, dims, init}, rest}` | Same shape; `init` is `( assign constant_expression )?`. |
| `list_of_cross_items` | `-> {first, second, rest}` | Cross requires ≥2 items per LRM A.2.11 — `first` and `second` are required, `rest` is the trailing iteration. |
| `list_of_checker_port_connections` (2 branches) | per-branch `{kind, items: {first, rest}}` | Kind labels: `"ordered"` / `"named"`. |
| `list_of_port_connections` (2 branches) | per-branch `{kind, items: {first, rest}}` | Kind labels: `"named"` / `"ordered"`. PEG ordered choice tries named first. |
| `cond_predicate` | `-> {first, rest}` | LRM A.6.7.1 `&&&`-separated chain of expression-or-cond_pattern values used in conditional statement predicates. |
| `cond_pattern` | `-> {expression, pattern}` | The `expr matches pattern` form used in conditional_statement guards. Drops `kw_matches`. |
| `expression_or_cond_pattern` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"cond_pattern"`. |
| `pattern_sv_2017` (6 branches) | per-branch typed | Kind labels: `"variable_capture"` (`{name}` — `.name` form, binds matched value) / `"wildcard"` (`.*`) / `"expression"` (`{body}`) / `"tagged"` (`{name, sub_pattern}` — tagged-union pattern) / `"ordered"` (`{patterns: {first, rest}}` — `'{p1, p2}` positional struct pattern) / `"named"` (`{entries: {first: {name, pattern}, rest}}` — `'{n1: p1, n2: p2}` keyed pattern). |
| `pattern_sv_2023` (7 branches) | per-branch typed | Same 6 kinds as sv_2017 plus `"parenthesized"` (`{body}` — `(pattern)` form added in LRM 2023 A.6.7.1 grammar expansion). |
| `assignment_pattern` | `-> {exprs: {first, rest}}` | The `'{expr, expr, ...}` assignment-pattern form (drops tick / braces). Mini-mixed-array on exprs. |
| `expression` (3 branches) | per-branch `{kind, body}` | Kind labels: `"base"` (typed expression_base) / `"inside"` (typed inside_expression) / `"conditional"` (typed conditional_expression). |
| `expression_base` (3 branches) | per-branch typed | Kind labels: `"tagged_union"` (`{body}`) / `"operand_chain"` (`{first, rest}` — binary-operator chain) / `"paren_op_assign"` (`{body}`). |
| `expression_operand` (3 branches) | per-branch typed | Kind labels: `"unary"` (`{op, attributes, primary}`) / `"inc_or_dec"` (`{body}`) / `"primary"` (`{body}`). |
| `expression_or_dist` | `-> {expr, dist}` | Wraps an expression with optional `dist { ... }` clause for constraint distribution. |
| `constant_expression` | `-> {first, rest, ternary}` | Binary-op chain with optional `?:` ternary tail. `ternary` is `[]` for non-ternary, `[<? attrs expr : expr>]` for ternary. |
| `constant_expression_operand` (2 branches) | per-branch typed | Kind labels: `"unary"` (`{op, attributes, primary}`) / `"primary"` (`{body}`). |
| `inside_expression_sv_2017` / `inside_expression_sv_2023` | `-> {expr, ranges}` | The `expr inside { range_list }` form per LRM A.6.7.1. sv_2017 uses open_range_list; sv_2023 uses range_list. |
| `conditional_expression` | `-> {predicate, attributes, then_expr, else_expr}` | Ternary `? :` form. The `&question` positive lookahead is preserved unchanged from source grammar. |
| `tagged_union_expression_sv_2017` / `tagged_union_expression_sv_2023` | `-> {name, value}` | `tagged member_name [value]` form. sv_2023 uses `primary` for value (more restrictive); sv_2017 uses `expression`. |
| `primary_literal` (4 branches) | per-branch `{kind, body}` | Kind labels: `"number"` / `"time_literal"` / `"unbased_unsized_literal"` (`'0` / `'1` / `'x` / `'z`) / `"string_literal"`. |
| `binary_operator` (29 branches) | per-branch `{kind}` (bare) | Kind labels: `"plus"` / `"minus"` / `"star"` / `"slash"` / `"percent"` / `"equal"` / `"not_equal"` / `"case_equal"` / `"case_not_equal"` / `"wildcard_equal"` / `"wildcard_not_equal"` / `"logical_and"` / `"logical_or"` / `"power"` / `"less_than"` / `"less_equal"` / `"greater_than"` / `"greater_equal"` / `"bitwise_and"` / `"bitwise_or"` / `"bitwise_xor"` / `"reduction_xnor_alt"` / `"reduction_xnor"` / `"shift_right"` / `"shift_left"` / `"arithmetic_shift_right"` / `"arithmetic_shift_left"` / `"implies"` / `"iff_arrow"`. |
| `unary_operator` (11 branches) | per-branch `{kind}` (bare) | Kind labels: `"plus"` / `"minus"` / `"bang"` / `"tilde"` / `"bitwise_and"` / `"reduction_nand"` / `"bitwise_or"` / `"reduction_nor"` / `"bitwise_xor"` / `"reduction_xnor"` / `"reduction_xnor_alt"`. |
| `primary_sv_2017` (15 branches) | per-branch typed | Kind labels: `"literal"` / `"call"` / `"hierarchical"` (`{scope, name, select}`) / `"empty_array_concat"` / `"multiple_concat"` (`{body, select}`) / `"concat"` / `"let"` / `"paren"` / `"cast"` / `"assign_pattern"` / `"streaming_concat"` / `"sequence_method"` / `"this"` (bare) / `"system_dollar"` (bare) / `"null_class_assign"` (`{local_n, scope}` — rare LRM null:= form). |
| `primary_hier_scope_prefix` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"class_qualifier"` / `"package_scope"`. Helper rule extracted from inline `( kw_class_qualifier \| non_typedef_package_scope )?`. |
| `instance_or_class_scope` (NEW; 2 branches) | per-branch typed | Kind labels: `"instance"` (`{handle}`) / `"class_scope"` (`{body}`). Helper rule extracted from inline `( implicit_class_handle dot \| class_scope )?`. |
| `constant_primary_sv_2017` (15 branches) | per-branch typed | Kind labels: `"literal"` / `"ps_parameter"` (`{name, select}`) / `"specparam"` / `"genvar"` / `"formal_port"` / `"enum"` (`{scope, name}`) / `"multiple_concat"` / `"concat"` / `"function_call"` / `"let"` / `"paren"` / `"cast"` / `"assign_pattern"` / `"type_reference"` / `"null"` (bare). |
| `enum_id_scope_prefix` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"package_scope"` / `"class_scope"`. Helper rule extracted from inline `( non_typedef_package_scope \| class_scope )?` in constant_primary_sv_2017's enum branch. |
| `primary_sv_2023` (15 branches) | per-branch typed | Same 15 kind labels as primary_sv_2017 except `"call"` adds optional `select` field — `{body, select}` (LRM 2023 allows `f()[0]` array-indexed call). Reuses `primary_hier_scope_prefix` and `instance_or_class_scope` helpers. |
| `constant_primary_sv_2023` (16 branches) | per-branch typed | sv_2017's 15 kinds plus `"empty_array_concat"` (`{body}`) added per LRM 2023; also `"function_call"` adds optional `select` field. Reuses `enum_id_scope_prefix` helper. |
| `attr_spec` | `-> {name, value}` | LRM A.9.1 attribute spec: `name [= value]`. `value` is `[]` for bare attribute, `[<assign, expr>]` when explicit. |
| `cast` / `constant_cast` | `-> {type, body}` | LRM cast form `type'(expr)`. Drops tick / parens. |
| `concatenation` / `constant_concatenation` | `-> {first, rest}` | LRM `{e1, e2, ...}`. Mini-mixed-array. Drops braces. |
| `multiple_concatenation` / `constant_multiple_concatenation` | `-> {count, body}` | LRM `{N{...}}` replication. |
| `streaming_concatenation` | `-> {op, slice_size, body}` | LRM A.8.1 `<<size{...}>>` / `>>{...}` form. `op` is `<<` or `>>`. `slice_size` is `[]` for default-bit-stream. |
| `call_primary` (6 branches) | per-branch `{kind, body}` | Kind labels: `"split_direct_callable_method"` / `"class_scoped_tf"` / `"plain_tf"` / `"tf"` / `"direct_callable_method"` / `"system_tf"`. |
| `casting_type` (5 branches) | per-branch typed | Kind labels: `"simple_type"` (`{body}`) / `"constant_primary"` (`{body}` — width-cast `N'(expr)`) / `"signing"` (`{body}`) / `"string"` (bare) / `"const"` (bare). |
| `bit_select` | `-> {body}` | Quantified iteration `( lbrack bit_select_expression rbrack )*` — multi-dimensional bit select. |
| `system_tf_call` (3 branches) | per-branch typed | Kind labels: `"args"` (`{name, args}` — `$display(...)` general form) / `"data_type"` (`{name, data_type, expr}` — `$cast(type, expr)`) / `"expr_clocking"` (`{name, first_expr, rest_exprs, clocking}` — `$rose` / `$past` etc.). |
| `select` | `-> {member_chain, tail}` | LRM A.8.5 select form. `member_chain` is the optional `.foo.bar` dereference (each segment optionally followed by a bit_select); `tail` is the optional bracket-index portion typed via `select_tail` helper. |
| `select_tail` (NEW; 2 branches) | per-branch typed | Kind labels: `"part_range"` (`{body}` — `[N:M]`) / `"bit_select"` (`{bits, range}` — multi-dim with optional trailing range). Helper extracted from inline parens-Or to dodge task #38. |
| `constant_select` | `-> {member_chain, tail}` | Parallel to select; uses `constant_select_tail` helper. |
| `constant_select_tail` (NEW; 2 branches) | per-branch typed | Same kinds as select_tail with constant_* sub-rules. Helper extracted from inline parens-Or. |
| `constant_range` | `-> {lo, hi}` | LRM `[lo:hi]` part-range. Drops colon. |
| `constant_range_expression` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"part_select_range"`. |
| `simple_type` (4 branches) | per-branch `{kind, body}` | Kind labels: `"integer"` / `"non_integer"` / `"ps_type"` / `"ps_parameter"`. Used by casting_type.kind=="simple_type". |
| `range_expression` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"part_select_range"`. |
| `part_select_range` / `constant_part_select_range` (2 branches each) | per-branch `{kind, body}` | Kind labels: `"range"` / `"indexed_range"`. |
| `indexed_range` / `constant_indexed_range` (2 branches each) | per-branch typed | Kind labels: `"plus_indexed"` (`{base, width}` — `[base+:width]`) / `"minus_indexed"` (`{base, width}` — `[base-:width]`). LRM 1800-2017 §11.5.1. |
| `dist_list` | `-> {first, rest}` | LRM dist clause iteration. Mini-mixed-array. |
| `dist_item_sv_2017` | `-> {value, weight}` | Single value-range with optional weight. |
| `dist_item_sv_2023` (2 branches) | per-branch typed | Kind labels: `"value"` (`{value, weight}`) / `"default"` (`{weight}` — `default :/ weight` form per LRM 2023). |
| `dist_weight` (2 branches) | per-branch typed | Kind labels: `"equal"` (`{weight}` — `:=` operator, equal share) / `"proportional"` (`{weight}` — `:/` operator, divided weight). |
| `range_list_sv_2023` / `open_range_list_sv_2017` | `-> {first, rest}` | Mini-mixed-array. |
| `value_range_sv_2017` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"range"`. |
| `value_range_sv_2023` (5 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"range"` / `"dollar_lo"` (`[$:expr]`) / `"dollar_hi"` (`[expr:$]`) / `"tolerance"` (`[expr +/- expr]`). LRM 2023 expansion. |
| `array_method_name` (5 branches) | per-branch typed | Kind labels: `"method_identifier"` (`{body}` — user-defined method) / `"unique"` / `"and"` / `"or"` / `"xor"` (bare; LRM-reserved array-builtin method names per A.2.10). |
| `class_new` (2 branches) | per-branch typed | Kind labels: `"constructor"` (`{scope, args}` — `new(args)` with optional class scope) / `"copy"` (`{source}` — `new other` shallow-copy form). |
| `dynamic_array_new` | `-> {size, init}` | LRM `new[size]` or `new[size](init)` form. |
| `empty_unpacked_array_concatenation` | `-> {kind: "empty_unpacked_array_concat"}` | The `'{}` empty-unpacked-array-concat literal per LRM 2023. |
| `join_keyword` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"join"` / `"join_any"` / `"join_none"`. Used by `par_block.join` from SV-Slice-33. |
| `slice_size` (2 branches) | per-branch `{kind, body}` | Kind labels: `"simple_type"` / `"constant_expression"`. Used by streaming_concatenation.slice_size. |
| `stream_concatenation` | `-> {body}` | Quantified-of-Quantified iteration `( stream_expression ( comma stream_expression )* )*`. |
| `stream_expression` | `-> {expr, with_clause}` | Optional `with [array_range_expression]` per LRM A.8.1. |
| `stream_operator` (2 branches) | per-branch `{kind}` (bare) | Kind labels: `"shift_right"` (`>>`) / `"shift_left"` (`<<`). |
| `charge_strength` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"small"` / `"medium"` / `"large"`. |
| `cycle_delay` (3 branches) | per-branch `{kind, body}` | Kind labels: `"number"` / `"identifier"` / `"expression"` (`##N` form). |
| `cycle_delay_const_range_expression` (2 branches) | per-branch typed | Kind labels: `"range"` (`{lo, hi}`) / `"dollar_hi"` (`{lo}` — `[lo:$]` form). |
| `delay_control` (2 branches) | per-branch `{kind, body}` | Kind labels: `"value"` (`#N`) / `"mintypmax"` (`#(...)`). |
| `delay_or_event_control` (3 branches) | per-branch typed | Kind labels: `"delay"` (`{body}`) / `"event"` (`{body}`) / `"repeat"` (`{count, control}` — `repeat (count) event_control`). |
| `delay_value` (5 branches) | per-branch typed | Kind labels: `"unsigned_number"` / `"real_number"` / `"ps_identifier"` / `"time_literal"` (each `{body}`) / `"step"` (bare — `1step`). |
| `event_control_sv_2017` (5 branches) | per-branch typed | Kind labels: `"event"` (`{body}` — `@event_id`) / `"expression"` (`{body}` — `@(expr)`) / `"wildcard"` (bare — `@*`) / `"wildcard_alt"` (bare — `@(*)` LRM-alt syntax) / `"sequence"` (`{body}` — `@seq_id`). |
| `event_control_sv_2023` (3 branches) | per-branch typed | Kind labels: `"clocking"` (`{body}` — clocking_event prefix) / `"wildcard"` / `"wildcard_paren"` (bare). LRM 2023 simplifies the event_control set. |
| `event_expression_primary` (3 branches) | per-branch typed | Kind labels: `"expression"` (`{edge, expr, iff}`) / `"sequence"` (`{body, iff}`) / `"paren"` (`{body}`). |
| `strength` (4 branches) | per-branch `{kind}` (bare) | Kind labels: `"supply"` / `"strong"` / `"pull"` / `"weak"`. |
| `class_constructor_prototype_sv_2017` / `class_constructor_prototype_sv_2023` | `-> {ports}` | Drops `kw_function`, `kw_new`, `semi`. `ports` is the optional `( lparen ... rparen )?` slot. |
| `clocking_decl_assign` | `-> {name, value}` | LRM A.6.10. `value` is `[]` or `[<assign, expr>]`. |
| `clocking_declaration` | `-> {default_keyword, name, event, items, end_label}` | Drops `kw_clocking`, `kw_endclocking`, `semi`. |
| `clocking_direction` (4 branches) | per-branch typed | Kind labels: `"input"` (`{skew}`) / `"output"` (`{skew}`) / `"input_output"` (`{input_skew, output_skew}`) / `"inout"` (bare). |
| `clocking_event_sv_2017` | `-> {body}` | The simple `@id` form. |
| `clocking_event_sv_2023` (3 branches) | per-branch `{kind, body}` | Kind labels: `"ps"` / `"hierarchical"` / `"expression"`. LRM 2023 expansion. |
| `clocking_item` (3 branches) | per-branch typed | Kind labels: `"default_skew"` (`{skew}`) / `"direction"` (`{direction, decls}`) / `"assertion"` (`{attributes, body}`). |
| `clocking_skew` (2 branches) | per-branch typed | Kind labels: `"edge"` (`{edge, delay}`) / `"delay"` (`{body}`). |
| `edge_identifier` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"posedge"` / `"negedge"` / `"edge"`. |
| `method_prototype` (2 branches) | per-branch `{kind, body}` | Kind labels: `"task"` / `"function"`. |
| `class_constructor_arg_sv_2023` (2 branches) | per-branch typed | Kind labels: `"tf_port_item"` (`{body}`) / `"default"` (bare; LRM 2023 explicit-default arg). |
| `class_constructor_arg_list_sv_2023` | `-> {first, rest}` | Mini-mixed-array. |
| `class_constructor_declaration_sv_2017` | `-> {class_scope, ports, decls, super_call, statements, end_label}` | Drops `kw_function`, `kw_new`, `semi`, `kw_endfunction`. |
| `class_constructor_declaration_sv_2023` | `-> {class_scope, ports, decls, super_call, statements, end_label}` | Parallel shape; uses `class_constructor_super_args` helper for the inner `( list_of_arguments \| kw_default )?` parens-Or in the super.new clause. |
| `class_constructor_super_args` (NEW; 2 branches) | per-branch typed | Kind labels: `"args"` (`{body}` — `super.new(args);`) / `"default"` (bare — `super.new(default);`). Helper rule extracted from inline parens-Or to dodge task #38. |
| `tf_port_list` | `-> {first, rest}` | Function/task port list. Mini-mixed-array. |
| `tf_port_item` | `-> {attributes, direction, var_keyword, data_type, port_spec}` | Single port-item per LRM A.2.7. |
| `tf_port_direction_sv_2017` (2 branches) | per-branch typed | Kind labels: `"port_direction"` (`{body}`) / `"const_ref"` (bare). |
| `tf_port_direction_sv_2023` (2 branches) | per-branch typed | Kind labels: `"port_direction"` (`{body}`) / `"ref"` (`{const_keyword, static_keyword}` — LRM 2023 expanded ref form). |
| `function_prototype_sv_2017` | `-> {return_type, name, ports}` | Drops `kw_function`. |
| `function_prototype_sv_2023` | `-> {dynamic_override, return_type, name, ports}` | Adds LRM 2023 dynamic_override slot. |
| `task_prototype_sv_2017` / `task_prototype_sv_2023` | `-> {[dynamic_override,] name, ports}` | Parallel to function_prototype. |
| `let_port_item` | `-> {attributes, type, name, dims, init}` | LRM A.2.8 let port. |
| `let_port_list` | `-> {first, rest}` | Mini-mixed-array. |
| `net_decl_assignment` | `-> {name, dims, init}` | Drops `assign` (init has the body). |
| `variable_decl_assignment` (3 branches) | per-branch typed | Kind labels: `"variable"` (`{name, dims, init}`) / `"dynamic_array"` (`{name, unsized_dim, dims, init}`) / `"class"` (`{name, init}` — `class_var = new` form). |
| `net_lvalue` (3 branches) | per-branch typed | Kind labels: `"name"` (`{name, select}`) / `"concatenation"` (`{items: {first, rest}}`) / `"pattern"` (`{type, body}`). |
| `variable_lvalue` (4 branches) | per-branch typed | Kind labels: `"name"` (`{scope, name, select}` — scope uses NEW `variable_lvalue_scope` helper) / `"concatenation"` (`{items: {first, rest}}`) / `"pattern"` (`{type, body}`) / `"streaming_concatenation"` (`{body}`). |
| `variable_lvalue_scope` (NEW; 2 branches) | per-branch typed | Kind labels: `"instance"` (`{handle}` — implicit_class_handle dot prefix) / `"package_scope"` (`{body}`). Helper extracted from inline `( implicit_class_handle dot \| non_typedef_package_scope )?` to dodge task #38. |

## Sub-rules with implicit defaults

Rules that have no explicit annotation default to their grammar-shape envelope (see [Parse Content Variants](parse-content-variants.md)). The default is documented at the rule level in `grammars/systemverilog.ebnf` comments where the default is non-obvious.

## Unannotated-on-purpose rules

Some rules will remain un-annotated by design — typically utility / helper rules whose envelope shape is the most useful representation, or rules whose typed shape would be redundant with their parent rule's shape.

| Rule | Reason |
|---|---|
| _(none yet)_ | _(none yet)_ |

Each row added here will cite the slice that decided the rule should remain un-annotated.

## How to read the annotation column

The annotation column shows the EBNF `-> ...` clause from `grammars/systemverilog.ebnf`. The reference grammar for the annotation language is:

- `$N` — positional reference to the Nth body element (1-indexed).
- `$N.field` — member access on a typed sub-rule shape.
- `{field: value, ...}` — typed object literal.
- `[v1, v2, ...]` — array literal.
- `[$N**]` — flatten-spread an array-shaped reference.
- `true` / `false` / `null` — boolean / null scalars.
- `@transform` — typed numeric value via `str::parse::<TYPE>`-style transform.
- `"text"` — string literal.

See `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` for the full annotation-language grammar.
