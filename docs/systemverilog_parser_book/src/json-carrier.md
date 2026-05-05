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
