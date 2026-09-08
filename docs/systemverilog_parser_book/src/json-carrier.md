# The Json Carrier

This chapter is a flat reference table of every `systemverilog.ebnf` rule that carries a `-> ...` return annotation, and the JSON shape that annotation produces.

> **Note:** The systemverilog return-annotation campaign is **in early phase**. Most rules are currently un-annotated and produce the recursive-envelope shape described in [AST Envelope Structure](ast-envelope.md). This table grows as the campaign progresses.

## Currently annotated rules

| Rule | Annotation | JSON shape produced |
|---|---|---|
| `systemverilog_file` | `-> {type: "systemverilog_file", source_text: $2}` | `{"type": "systemverilog_file", "source_text": <source_text-shape>}` — root JSON object for any `sv_2017` / `sv_2023` parse. `source_text` field is a flat array of `source_text_item` shapes (since SV-Slice-2). |
| `systemverilog_parseable_file` | `-> {type: "systemverilog_parseable_file", items: $2}` | `{"type": "systemverilog_parseable_file", "items": <parseable_source_item*-shape>}` — alternative entry rule for the parseable-source profile. `items` field carries the array of parseable source items in their raw envelope shape. |
| `source_text` | `-> [$1**]` | `[<source_text_item-typed-shape>, ...]` — flat array of typed source-text items via flatten-spread. Each item is a `{kind: "<name>", body: <envelope>}` typed object (per SV-Slice-3). |
| `source_text_item` (7 branches) | per-branch `{kind: "<name>", body: $1}` (or `{kind: "semi"}` for branch 6) | Typed object with `kind` discriminator: `"description"`, `"local_parameter_declaration"`, `"parameter_declaration"`, `"package_import_declaration"`, `"timeunits_declaration"`, `"compiler_directive"`, `"semi"`. The `body` field carries the matched sub-rule's raw envelope OR a typed sub-rule shape if that sub-rule is itself annotated. For `kind: "description"`, body is now itself typed (per SV-Slice-4). The `semi` branch carries no `body` since it's just a stray `;`. Trailing `semi` dropped in branches 1 and 2 (annotation references `$1` only). **The former 8th branch `comment_only_source_region` was removed (GRAMMAR-WELLFORMED.H.12.5.3): it was engine-shadowed-dead — the layout skipper consumes comments as leading trivia before this rule's branch is tried, so a comment-only region is always absorbed as layout and the `comment_only_source_region` kind was never emitted in any AST (a comment-only file parses to an empty `source_text`). Removing it is accept-identical — no schema/release bump — and narrows the declared union to the kinds that can actually occur. |
| `description` (8 branches) | per-branch `{kind: "<name>", body: $1}` for single-element branches; `{kind: "<name>", attributes: $1, body: $2}` for multi-element branches with `attribute_instance*` prefix | Typed object with `kind` discriminator: `"module_declaration"`, `"udp_declaration"`, `"interface_declaration"`, `"program_declaration"`, `"package_declaration"`, `"package_item"`, `"bind_directive"`, `"config_declaration"`. The `attributes` field (only on `package_item` / `bind_directive` branches) carries the leading `attribute_instance*` iteration. The `body` field carries the matched sub-rule's raw envelope (per-rule typing of `module_declaration`, etc. is a follow-up slice). |
| `compiler_directive` (2 branches; **schema 29**) | passthrough / `{kind: "opaque", text: $2}` | **Since parser release `1.0.217` (`SV-CORPUS-GRAD.3.39`) the body is a TYPED node.** Branch 1 passes through one of the thirteen `*_directive` nodes below (`{kind: "timescale", unit, precision}`, `{kind: "line", number, filename, level}`, …); branch 2 — any other backtick line at file top level (`` `include ``, `` `define ``, the conditional family, a text-macro use, a vendor directive) — is `{kind: "opaque", text: "<the line>"}`. ⚠️ Through `1.0.216` `body` was the directive LINE as one string (`` "`timescale 1ns / 1ps" ``); a consumer that read it as a string must now switch on `body.kind`, and `body.kind == "opaque"` carries the old string as `text`. The thirteen typed names are refused when their arguments do not match the clause's syntax box (`` `timescale frump `` no longer parses). See [Typed Compiler Directives](#typed-compiler-directives-schema-29). |
| `in_scope_compiler_directive` (7 branches; schema 29) | passthrough | The in-scope whitelist (`.3.14b`) as the same typed nodes: `celldefine`, `endcelldefine`, `undefineall` (SV-only), `undef`, `timescale`, `pragma`, `line`. No `opaque` branch in scope, by design. `non_port_module_item` and `class_item` wrap it as `{kind: "compiler_directive", body: <typed node>}` — the same node the top-level container carries. ⚠️ A consumer that exhaustively matches module-item or class-item kinds must carry a `compiler_directive` arm (since `1.0.178`). |
| `resetall_directive` / `celldefine_directive` / `endcelldefine_directive` / `nounconnected_drive_directive` / `end_keywords_directive` / `undefineall_directive` (schema 29) | `-> {kind: "<name>"}` | The argument-less directives (IEEE 1800-2017 §22.3, §22.10, §22.9, §22.14, §22.5.3). `undefineall_directive` is `@profiles: ["sv_2017", "sv_2023"]`; under `verilog_2005` the same text is a text-macro use and lands as `{kind: "opaque"}`. |
| `undef_directive` (schema 29) | `-> {kind: "undef", name: $2}` | `name` is an `identifier` node (`{body}`; simple or escaped — §22.5.2 Syntax 22-4). |
| `timescale_directive` (schema 29) | `-> {kind: "timescale", unit: $2, precision: $4}` | Both slots are `directive_time_literal` nodes `{value: "1" \| "10" \| "100", unit: {kind: "s" \| "ms" \| "us" \| "ns" \| "ps" \| "fs"}}` (§22.7 Syntax 22-6; the magnitude is SYNTAX here — "the valid integers are 1, 10, and 100" — while `timeunit 2ps;` stays derivable through Annex A's own production). `unit.unit` reuses the `time_unit` node kinds. The precision-ordering rule ("at least as precise", §22.7 prose) is NOT judged: `` `timescale 1ps/1ns `` parses. |
| `directive_time_literal` / `directive_time_magnitude` (schema 29) | `{value: $1, unit: $2}` / `-> $2` | `value` is the magnitude text; white space between magnitude and unit is admitted (`1 ns`, the clause's own example). |
| `default_nettype_directive` / `default_nettype_value` / `default_nettype_none_word` (schema 29) | `{kind: "default_nettype", value: $2}` / per-branch `{kind: "<value>"}` | `value.kind` is one of §22.8 Syntax 22-7's eleven: `wire tri tri0 tri1 wand triand wor trior trireg uwire none` — deliberately NOT `net_type` (no `supply0` / `supply1`). |
| `unconnected_drive_directive` / `unconnected_drive_value` (schema 29) | `{kind: "unconnected_drive", value: $2}` / `{kind: "pull0"} \| {kind: "pull1"}` | §22.9: "takes one of two arguments — pull1 or pull0". `nounconnected_drive_directive` takes none. |
| `pragma_directive` (2 branches, `@branch_policy: ordered`; schema 29) | `{kind: "pragma", name: $2, expressions: [] \| $3}` | `name` is the `pragma_name` string (a `simple_identifier`, keywords admitted — `` `pragma protect begin `` has `begin` as a pragma_keyword). Branch 1 is the bare-name form bounded to its LINE (`pragma_line_end`), so `` `pragma once `` followed by an identifier-led item on the next line does not eat that item; branch 2 is the comma-separated `pragma_expression_list` (§22.11 Syntax 22-8). Ordered choice because the grammar's default longest-match tournament would have picked branch 2 whenever the next line began with an identifier. |
| `pragma_expression_list` / `pragma_expression` / `pragma_keyword` / `pragma_name` (schema 29) | `[$1, $2::2*]` / per-branch `{kind: "assignment", keyword, value}` \| `{kind: "keyword", name}` \| `{kind: "value", value}` / passthrough strings | A bare identifier is a `keyword` (the LRM lists `pragma_keyword` first); `pragma_value`'s `identifier` arm is therefore reached only on the right of `=` (`key_method = RSA`). |
| `pragma_value` (4 branches; schema 29) | per-branch `{kind: "group", items}` \| `{kind: "number", body}` \| `{kind: "string", body}` \| `{kind: "identifier", body}` | `group.items` is a nested `pragma_expression_list`; `number.body` is the `number` node (`{kind: "integral" \| "real", body}`); `string.body` a `string_literal` node (`{kind: "double_quoted", body}` with the quotes). |
| `line_directive` / `line_number` / `line_level` / `line_end` (schema 29) | `{kind: "line", number: $2, filename: $3, level: $4}` / `-> $2` / `-> $2` / (regex, no node) | §22.12 Syntax 22-9: `number` a positive integer (`[1-9][0-9]*`, as text), `filename` a `string_literal` node, `level` `"0"` \| `"1"` \| `"2"`; `line_end` is the empty-matching regex that asserts "only white space may appear on the same line … comments are not allowed" — the one directive the clause binds to its line. |
| `begin_keywords_directive` / `keywords_version_specifier` (+ `_sv_2017` / `_sv_2023` / `_v2005`) (schema 29) | `{kind: "begin_keywords", version: $2}` / `{kind: "double_quoted", body: $2}` | `version` carries the quoted literal exactly as `string_literal` does. One profiled token per edition holds that edition's list: IEEE 1800-2017 Syntax 22-10's eight, IEEE 1800-2023's nine (adds `"1800-2023"`), IEEE 1364-2005 Syntax 19-10's four — so `` `begin_keywords "1800-2023" `` parses under `sv_2023` and REJECTS under `sv_2017` ("it shall be an error if an implementation does not recognize the version_specifier"). |
| `lrm_directive_name` / `lrm_directive_name_shared` / `lrm_directive_name_sv_only` / `opaque_directive_line` (schema 29) | (lookahead subject) / `-> $2` | `lrm_directive_name` is referenced only as `!lrm_directive_name` — the guard that keeps the thirteen typed names out of the opaque residue, so a malformed typed directive REJECTS instead of falling back to the 1.0.216 tolerance. `opaque_directive_line` is the 1.0.216 blanket token, now reachable only past that guard. |
| `kw_dir_*` (14 tokens; schema 29) | (raw token) | `` `resetall `` … `` `end_keywords `` and `` `undefineall `` (SV-only), each `` trivia /`<name>\b/ ``; `default_nettype_none_word` spells `none`, which is not a reserved word. |
| `expression_or_dist` | `-> {expr: $1, dist: $2}` (the dist group is `kw_dist lbrace dist_list rbrace`) | **Shape changed in parser release `1.0.179` (schema `19` → `20`).** `{"expr": <expression-shape>, "dist": []}` when there is no `dist` clause; otherwise `dist` is the **four**-element array `[[trivia,"dist"], {kind:"lbrace"}, <dist_list>, {kind:"rbrace"}]` — the literal IEEE braces are nodes. Before `1.0.179` it was a two-element array and the item list was mis-parsed; see [The `dist` Constraint Operator](#the-dist-constraint-operator). |
| `attribute_instance` | `-> [$2, $3::2*]` | Flat array `[<attr_spec shape>, <attr_spec shape>, ...]`. Drops the `attr_open` (`(*`) and `attr_close` (`*)`) delimiters and the comma separators between attr_specs. Slice 58 audit replaced the prior `{first, rest}` shape with the flat extraction-spread form per `grammars/return_annotation.ebnf` line 158's self-application. |
| `module_declaration_sv_2017` (5 branches) | per-branch typed shapes, see contract section "Release 1.0.6 / Contract 1.0.6 Highlights" for the full annotation source | Typed object with `kind` discriminator: `"ansi"` / `"nonansi"` / `"wildcard"` / `"extern_nonansi"` / `"extern_ansi"`. Single-form branches expose `header / timeunits / items / end_label`. The wildcard branch additionally exposes `attributes / keyword / lifetime / name`. Extern branches expose only `header`. |
| `module_declaration_sv_2023` (5 branches) | per-branch typed shapes (same kind set as sv_2017) | Identical kind discriminator and field names as sv_2017. Wildcard branch's positional indices shift due to `dot star` (2 tokens) vs `dot_star` (1 token); user-visible AST is identical to sv_2017. |
| `module_keyword` (2 branches) | `-> {kind: "module"}` / `-> {kind: "macromodule"}` | Typed object with `kind` discriminator. Drops the keyword token (redundant with `kind`). When referenced by parent rules (e.g. module_ansi_header.keyword), the typed shape propagates automatically. |
| `lifetime` (2 branches) | `-> {kind: "static"}` / `-> {kind: "automatic"}` | Typed object. When `(lifetime)?` is matched, consumers see `{kind: "static"}` / `{kind: "automatic"}`. When un-matched, `[]` (existing convention). |
| `module_ansi_header` | `-> {attributes, keyword, lifetime, name, imports, parameters, ports}` | 7 named fields. `keyword:` is itself typed (per `module_keyword`); `lifetime:` is itself typed when matched (per `lifetime`); `attributes`/`imports`/`parameters`/`ports` are quantified or optional (consumer handles `[]` for empty). `name:` carries raw `module_identifier` envelope. |
| `module_nonansi_header` | `-> {attributes, keyword, lifetime, name, imports, parameters, ports}` | Same field names as module_ansi_header. Only `ports:` source rule differs (`list_of_ports` vs `(list_of_port_declarations)?`); the typed shape is identical for consumers. |
| `ansi_port_header` | `-> $1` (un-annotated named rule; `net_port_header \| interface_port_header`) | **Since 1.0.152 (SV-0014, `SV-AST-SHAPE-FIDELITY.1`).** Passthrough of whichever port-header form matched: `net_port_header` `{direction, port_type}` or `interface_port_header` `{kind:"named", name, modport}`. This named rule was introduced to fix an AST-shape corruption — see `ansi_port_declaration`. |
| `ansi_port_declaration` (branch 1, `net_or_interface`) | `-> {kind: "net_or_interface", header: $1, name: $2, dims: $3, default: $4}` | A typed ANSI module port (`input logic a`, `output logic [7:0] b`, `input logic c = 0`). `header:` is the clean `ansi_port_header` shape (`net_port_header` `{direction, port_type}` or `interface_port_header` `{kind:"named", …}`), `[]` when the header is omitted (the port inherits the previous port's direction/type per IEEE 1800 §23.2.2.3); `name:` is the `port_identifier` `{body:…}`; `dims:` is the (possibly empty) unpacked-dimension list; `default:` is the optional `= constant_expression`, `[]` when absent. **⚠️ Schema 6 → 7 (1.0.152, ledger `SV-0014`):** at releases `1.0.0`–`1.0.151` `header:` bound an inline alternation directly, corrupting the AST — every typed ANSI port emitted **three `<invalid_sequence_access>` sentinels** in its `header` sub-shape (`net_port_header`'s real `{direction, port_type}` mis-nested as `header.header`). `1.0.152` lifts the alternation into the named `ansi_port_header` rule; `header:` is now clean. Consumers reading a typed ANSI port's `header` must adopt the `{direction, port_type}` shape. See [Schema Versioning](schema-versioning.md). |
| `forward_type_keyword` | `-> $1` (un-annotated named rule; `kw_enum \| kw_struct \| kw_union`) | **Since 1.0.153 (SV-0015, `SV-AST-SHAPE-FIDELITY.2.1`).** Passthrough of whichever forward-typedef keyword matched, as the raw fused-token `[trivia, "enum"]` / `[trivia, "struct"]` / `[trivia, "union"]` sequence (the grammar's convention for a direct keyword field). This named rule was introduced to fix an AST-shape corruption — see `type_declaration`. |
| `type_declaration_sv_2017` / `_sv_2023` (branch 6, `forward`) | `-> {kind: "forward", keyword: $2, name: $3}` | A forward typedef (`typedef enum e_t;` / `struct` / `union`, or the keyword-less `typedef e_t;`). `keyword:` is the `forward_type_keyword?` shape — the clean fused-token `[trivia, "enum"]` when present, `[]` when absent (bare form); `name:` is the `declared_type_identifier` `{body:…}`. **⚠️ Schema 7 → 8 (1.0.153, ledger `SV-0015`):** at releases `1.0.0`–`1.0.152` `keyword:` bound an inline alternation directly, corrupting the AST — a forward typedef with a keyword emitted **3–4 `<invalid_sequence_access>` sentinels** in its `keyword` sub-shape (branch-1's `class_alias` annotation mis-recursed onto the `keyword` slot). `1.0.153` lifts the alternation into the named `forward_type_keyword` rule; `keyword:` is now clean. See [Schema Versioning](schema-versioning.md). |
| `base_class_type_head` | `-> $1` (un-annotated named rule; `scoped_base_class_type_identifier \| known_unscoped_base_class_type_identifier \| known_unscoped_base_class_type_parameter_identifier`) | **Since 1.0.154 (SV-0016, `SV-AST-SHAPE-FIDELITY.2.2`).** Passthrough of whichever base-class-head form matched: `scoped_base_class_type_identifier` `{scope, name}` (package-scoped base, `pkg::B`) or a bare `class_identifier` `{body:…}` (declared class / type-parameter base). This named rule was introduced to fix an AST-shape corruption — see `base_class_type`. |
| `base_class_type` (`extends` base) | `-> {head: $1, params: $2, scope_chain: $3}` | The base class in a `class D extends …;` clause. `head:` is the clean `base_class_type_head` shape (`{scope, name}` for a package-scoped base, `{body:…}` for a bare base); `params:` is the optional `parameter_value_assignment` (`[]` when absent); `scope_chain:` is the optional trailing `::class` chain (`[]` when absent). **⚠️ Schema 8 → 9 (1.0.154, ledger `SV-0016`):** at releases `1.0.0`–`1.0.153` `head:` bound an inline 3-way alternation directly, corrupting the AST — a package-scoped base (`class D extends pkg::B;`, ubiquitous in SV/UVM) emitted **two `<invalid_sequence_access>` sentinels** in its `params`/`scope_chain` sub-shapes (the self-annotated scoped branch's `{scope,name}` re-recursed the rule's own annotation onto the following slots); the declared-bare `extends B;` was clean. `1.0.154` lifts the alternation into the named `base_class_type_head` rule (mirroring the already-correct sibling `class_type`); `params`/`scope_chain` are now the clean `[]`. Consumers reading a base-class extends' `params`/`scope_chain` must adopt the clean `[]` shape. See [Schema Versioning](schema-versioning.md). |
| `interface_class_type_head` | `-> $1` (un-annotated named rule; `scoped_interface_class_type_identifier \| known_unscoped_interface_class_type_identifier`) | **Since 1.0.156 (SV-0018, `SV-AST-SHAPE-FIDELITY.2.5`).** Passthrough of whichever interface-class-type-head form matched: `scoped_interface_class_type_identifier` `{scope, name}` (package-scoped interface class, `pkg::IC`) or a bare `class_identifier` `{body:…}` (declared interface class). This named rule was introduced to fix an AST-shape corruption — see `interface_class_type`. |
| `interface_class_type` (`implements`) | `-> {name: $1, params: $2}` | An interface class named in a `class C implements …;` clause (and `interface class … extends …;`). `name:` is the clean `interface_class_type_head` shape (`{scope, name}` for a package-scoped interface class, `{body:…}` for a bare one); `params:` is the optional `parameter_value_assignment` (`[]` when absent). **⚠️ Schema 10 → 11 (1.0.156, ledger `SV-0018`):** at releases `1.0.0`–`1.0.155` `name:` bound an inline 2-way alternation directly, corrupting the AST — a class implementing a package-scoped interface class (`class C implements pkg::IC;`, common in UVM) emitted **one `<invalid_sequence_access>`** in the inner `name`'s `params` sub-shape (the self-annotated `scoped_interface_class_type_identifier` branch's `{scope,name}` re-recursed the rule's own `{name, params}` annotation onto the branch result). `1.0.156` lifts the alternation into the named `interface_class_type_head` rule (mirroring `base_class_type_head`); `params` is now the clean `[]`. Consumers reading an implemented interface class' `params` must adopt the clean `[]` shape. See [Schema Versioning](schema-versioning.md). |
| `nettype_scope_prefix` | `-> $1` (un-annotated named rule; `non_typedef_package_scope \| class_scope`) | **Since 1.0.157 (SV-0019, `SV-AST-SHAPE-FIDELITY.2.6`).** Passthrough of whichever nettype scope prefix matched: `non_typedef_package_scope` `{body:{kind:package,name}}` (`pkg::`) or `class_scope` `{body:…}`. Shared by both nettype rules (both branches). This named rule was introduced to fix an AST-shape corruption — see `net_type_declaration`. Order `non_typedef_package_scope \| class_scope` (distinct from the reversed `scoped_type_scope_prefix`). |
| `net_type_declaration_sv_2017` / `nettype_declaration_sv_2023` (2 branches each; **schema 31**) | data-type form `-> {data_type: $2, name: $3, with_clause: $4}` (sv_2017) / `-> {kind: "data_type", data_type: $2, name: $3, with_clause: $4}` (sv_2023); ALIAS form `-> {kind: "nettype", scope: $2, base: $3, name: $4}` (both editions) | A user-defined nettype declaration, IEEE 1800-2017 A.2.1.3 / §6.6.7. **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the ALIAS form `nettype wTsum nettypeid2;` — another name for an existing nettype — wins its tie in both edition twins, typed on `sv_2017` too, with its base store-gated.** Through `1.0.218` it was SHADOWED: both alternatives succeed on the LRM's own example (a declared nettype is a `type_name` fact, so the data-type form reads `wTsum` as a `known_unscoped_data_type`) and the longest-match tie went to declaration order, so the alias alternative never won and the AST called the base nettype a data type. `1.0.219` breaks the tie in the LRM's favour by a NEGATIVE LOOKAHEAD on the data-type alternative — it refuses a text that reads as an alias head, `[ scope ] <declared nettype> <identifier> ;` — with the ORDER left as shipped, and routes the alias base `net_type_identifier` (a bare `declaration_identifier` through `1.0.218`, one ungated twin per edition) to `checked_nettype_identifier \| wildcard_escape_nettype_identifier` (a declared nettype, or an undeclared name under `import pkg::*` — how `net_declaration` reads a net of a user-defined nettype, SV-PARSE-STRICT.2 / SV-0037). Two spellings were refuted before this one: reordering the UNGATED alternative read `nettype T wT;` (T a typedef) as an alias — the two-sided corpus sweep caught it on one file — and reordering the GATED one broke the certificate's own nettype prelude (the generator renders an unplanned declaration with the first of two equally cheap alternatives, so the prelude became an alias with an undeclared base and the union certificate read `UNKNOWN=2`). The alias alternative now wins exactly the alias declarations. ⚠️ One NARROWING: `nettype undeclared_t x;` with no wildcard import in scope — accepted through `1.0.218` down the ungated alias alternative, LRM-illegal (the base must be an existing nettype) — now REJECTS on both SV profiles; the corpus census read 0 accepting files down that alternative, and the pin `invalid_nettype_alias_undeclared_base.sv` holds it. `scope` is `[]` or the `nettype_scope_prefix` node, `base` the existing nettype's identifier node, `name` the new one. The data-type form is byte-identical across the release (`kind` absent on `sv_2017`). See [Typed Partial Alternatives](#typed-partial-alternatives-schema-31). The optional `with`-clause resolution-function reference may be package/class-scoped (`with pkg::f`); its scope element is the clean `nettype_scope_prefix` shape. **⚠️ Schema 11 → 12 (1.0.157, ledger `SV-0019`):** at releases `1.0.0`–`1.0.156` the double-nested `with`-clause `( non_typedef_package_scope \| class_scope )?` corrupted the AST — `nettype logic n with pkg::f;` emitted **three `<invalid_sequence_access>`** in the `with_clause` inner scope element (the `data_type`/`name`/`with_clause` slots) when the self-annotated `non_typedef_package_scope` branch won. `1.0.157` lifts the alternation into the shared named `nettype_scope_prefix` rule; the `with_clause` inner scope is now clean. The scoped-base `nettype pkg::base nt;` (br1) form was already clean. See [Schema Versioning](schema-versioning.md). |
| `block_data_type` (17 branches; **schema 32**) | branch 1 `-> {kind: "integer_vector", base: $1, signing: $2, dims: $3}` (**since `1.0.220`, `SV-CORPUS-GRAD.3.46` — `data_type`'s own template; through `1.0.219` the field set alone, no `kind`**); branches 2 / 4 / 5 / 8 `data_type`'s own templates `{kind: "integer_atom", base, signing}` / `{kind: "struct_union", header, packed_signing, members: {first, rest}, dims}` / `{kind: "enum", base_type, names, dims}` / `{kind: "virtual_interface", interface_keyword, name, params, modport}`; the twelve single-reference branches pass their child through | The data type of a block-scoped variable declaration (`block_data_declaration`'s `{kind: "block_data", body}`), the block split of `data_type` (IEEE 1800-2017 A.2.2.1; A.2.8). **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the four multi-element branches carry `data_type`'s own kinds, and since `1.0.220` (`SV-CORPUS-GRAD.3.46`, schema 32) the integer-vector branch does too** — every branch a consumer can meet on the `block_data` body now carries a `kind` or is a passed-through child, so the body is discriminated by `kind` exactly as `data_type` is. The vector form was the most common block declaration in the corpus without one: 712 nodes in 259 accepting files per SV lane, 130 in 73 on the 1364-2005 lane, every one moved by `1.0.220` and nothing else. |
| `class_scoped_call_prefix_head` | `-> $1` (un-annotated named rule; `scoped_class_scoped_call_prefix_identifier \| known_unscoped_class_scoped_call_class_identifier \| known_unscoped_class_scoped_call_interface_class_identifier \| known_unscoped_class_scoped_call_type_parameter_identifier`) | **Since 1.0.158 (SV-0020, `SV-AST-SHAPE-FIDELITY.2.7`).** Passthrough of whichever class-scoped-call-prefix head matched: `scoped_class_scoped_call_prefix_identifier` `{scope, name}` (a package-scoped class, `pkg::C`) or a bare class / interface-class / type-parameter identifier `{body:…}`. This named rule was introduced to fix an AST-shape corruption — see `class_scoped_call_prefix`. |
| `class_scoped_call_prefix` (`pkg::C::foo()`) | `-> {head: $1, params: $2, scope_chain: $4}` | The `<pkg>::<class>::` prefix of a class-scoped static method call. `head:` is the clean `class_scoped_call_prefix_head` shape (`{scope, name}` for a package-scoped class, `{body:…}` for a bare one); `params:` the optional `parameter_value_assignment` (`[]` when absent); `scope_chain:` the further `::class` hops (`[]` when none). **⚠️ Schema 12 → 13 (1.0.158, ledger `SV-0020`):** at releases `1.0.0`–`1.0.157` `head:` bound an inline 4-way alternation directly, corrupting the AST — `pkg::C::foo()` (reachable in statement + assignment-RHS context) emitted **two `<invalid_sequence_access>`** in the `params`/`scope_chain` slots (the self-annotated `scoped_class_scoped_call_prefix_identifier` branch's `{scope,name}` re-recursed the rule's own `{head, params, scope_chain}` annotation onto the branch result). `1.0.158` lifts the alternation into the named `class_scoped_call_prefix_head` rule (mirroring the in-file `class_scope_type_head`); `params`/`scope_chain` are now the clean `[]`. `SV-0013` gated the alt's branches but did not lift it — this completes that rule. Consumers reading a class-scoped call's `params`/`scope_chain` must adopt the clean `[]` shape. See [Schema Versioning](schema-versioning.md). |
| `class_scope_type_head` | `-> $1` (un-annotated named rule; `scoped_class_scope_identifier \| known_unscoped_class_scope_class_identifier \| known_unscoped_class_scope_interface_class_identifier \| known_unscoped_class_scope_type_parameter_identifier`) | **Since 1.0.155 (SV-0017, `SV-AST-SHAPE-FIDELITY.2.3`).** Passthrough of whichever class-scope-head form matched: `scoped_class_scope_identifier` `{scope, name}` (package-scoped, `pkg::B`) or a bare `class_identifier` `{body:…}`. This named rule was introduced to fix an AST-shape corruption — see `class_scope_type`. |
| `class_scope_type` | `-> {head: $1, params: $2, scope_chain: $3}` | The scoped class-scope head of a `C::D::E` chain (via `class_scope`). `head:` is the clean `class_scope_type_head` shape; `params:` / `scope_chain:` are the optional parameter-value-assignment / trailing `::class` chain (`[]` when absent). **⚠️ Schema 9 → 10 (1.0.155, ledger `SV-0017`):** at releases `1.0.0`–`1.0.154` `head:` bound an inline 4-way alternation directly, corrupting the AST — a scoped type reference emitted **`<invalid_sequence_access>`** in `params`/`scope_chain` (when the self-annotated `scoped_class_scope_identifier` branch won). `1.0.155` lifts the alternation into the named `class_scope_type_head` rule (mirroring the in-file `class_type_head`/`base_class_type_head`); `params`/`scope_chain` are now clean. See [Schema Versioning](schema-versioning.md). |
| `scoped_type_scope_prefix` | `-> $1` (un-annotated named rule; `class_scope \| non_typedef_package_scope`) | **Since 1.0.155 (SV-0017, `SV-AST-SHAPE-FIDELITY.2.3`).** Passthrough of whichever scope prefix matched for a scoped type identifier: `class_scope` `{body:…}` (`C::D::`) or `non_typedef_package_scope` (`pkg::`). This named rule was introduced to fix an AST-shape corruption — see `scoped_data_type_identifier`. Shared by both `scoped_block_type_identifier` and `scoped_data_type_identifier`. |
| `scoped_data_type_identifier` / `scoped_block_type_identifier` | `-> {scope: $1, type: $2, dims: $3}` | A package/class-scoped type reference (`C::D::E`), in `data_type` (`scoped_data_type`) and `block_data_type` contexts respectively. `scope:` is the clean `scoped_type_scope_prefix` shape; `type:` is the trailing `type_identifier` `{body:…}`; `dims:` is the (possibly empty) packed-dimension list. **⚠️ Schema 9 → 10 (1.0.155, ledger `SV-0017`):** at releases `1.0.0`–`1.0.154` `scope:` bound the inline alt `( class_scope | non_typedef_package_scope )` directly, corrupting the AST — a scoped type reference emitted **`<invalid_sequence_access>`** in `type`/`dims` (when the self-annotated `class_scope` branch won). `1.0.155` lifts the alternation into the shared named `scoped_type_scope_prefix` rule (one rule fixes both byte-identical rules); `type`/`dims` are now clean. See [Schema Versioning](schema-versioning.md). |
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
| `bind_target_instance_list` | `-> [$1, $2::2*]` | Flat array of `bind_target_instance` shapes. Slice 58 audit replaced `{first, rest}` with extraction-spread. |
| `interface_class_declaration` | `-> {name, parameters, extends, items, end_label}` | Single-sequence shape mirror to class_declaration. |
| `config_declaration` | `-> {name, local_params, design, rules, end_label}` | Single-sequence shape with config-specific fields. |
| `list_of_ports` | `-> [$2, $3::2*]` | Flat array of `port` shapes. Drops `lparen`/`rparen` and the comma separators (slice 58 audit). |
| `list_of_port_declarations` | `-> $2` | Transparent passthrough of the optional inner content (which is itself a 3-element envelope when populated). Drops parens. |
| `udp_port_list` | `-> [$1, $3, $4::2*]` | Flat array `[<output_port_identifier>, <input_port_identifier>, <input_port_identifier>, ...]`. Slice 58 audit flattened the prior `{output, inputs: {first, rest}}` into a single positional array; consumers identify the leading element as the output. |
| `udp_declaration_port_list` | `-> {output, inputs: [$3, $4::2*]}` | Slice 58 audit replaced `inputs: {first, rest}` with flat extraction-spread; `output` field is preserved. `inputs` is now a flat array of `udp_input_declaration` shapes. |
| `anonymous_program` | `-> {items}` | Drops kw_program/semi/kw_endprogram; surfaces only the items list. Reachable via package_item.kind = "anonymous_program". |
| `package_export_declaration` (2 branches) | per-branch typed | Kind labels: `"wildcard"` (`export *::*;`, drops content) / `"explicit"` (with `items: [$2, $3::2*]` — flat array of package_import_item per slice 58 audit). |
| `port` (2 branches) | per-branch typed | Kind labels: `"expression"` (positional port — `expr` may be `[]` for empty placeholder) / `"named"` (`.name(expr)` form — `name` is clean identifier string). |
| `port_direction` (4 branches) | per-branch `{kind}` | Kind labels: `"input"` / `"output"` / `"inout"` / `"ref"`. |
| `package_import_declaration` | `-> {items: [$2, $3::2*]}` | Drops kw_import/semi; surfaces import items as a flat array (slice 58 audit). |
| `package_import_item` (2 branches) | per-branch typed | Kind labels: `"explicit"` (`pkg::name` with `package` and `name` as clean identifier strings) / `"wildcard"` (`pkg::*` with only `package`). |
| `udp_body` (2 branches) | per-branch `{kind, body}` | Kind labels: `"combinational"` / `"sequential"`. |
| `udp_input_declaration` | `-> {attributes, identifiers}` | Drops kw_input. `identifiers` is the typed list_of_udp_port_identifiers `{first, rest}`. |
| `udp_output_declaration` (2 branches) | per-branch typed | Kind labels: `"wire"` (no `assign` clause) / `"reg"` (with optional default). |
| `combinational_body` | `-> {entries: {first, rest}}` | Drops kw_table/kw_endtable. Entries are the truth-table rows. |
| `sequential_body` | `-> {initial, entries: {first, rest}}` | Preserves optional initial statement. |
| `list_of_udp_port_identifiers` | `-> [$1, $2::2*]` | Flat array of port_identifier strings (slice 58 audit). |
| `combinational_entry` | `-> {inputs, output}` | Single truth-table row for combinational UDP. **Realized as of release `1.0.147` / schema `6` (ledger `SV-0009`); releases `1.0.66`–`1.0.146` emitted the raw `Sequence` envelope here due to a duplicate-`->` return-annotation regression.** |
| `sequential_entry` | `-> {inputs, current_state, next_state}` | Single state-transition row for sequential UDP. **Realized as of release `1.0.147` / schema `6` (ledger `SV-0009`); raw `Sequence` at `1.0.66`–`1.0.146` (same duplicate-`->` regression).** |
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
| `case_generate_item` (2 branches) | per-branch typed | Kind labels: `"expr_list"` (`{exprs: [$1, $2::2*], block}` — flat array of constant_expression labels per slice 58 audit) / `"default"` (`{block}` — drops `kw_default` and the optional `( colon )?` separator). |
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
| `function_body_declaration` (4 branches; **schema 30**) | per-branch typed — `{return_type: $1, name: $2, items: $4, statements: $5, end_label: $7}` / `{return_type: $1, name: $2, ports: $4, items: $7, statements: $8, end_label: $10}` / `{return_type: $1, scope: $2, name: $3, items: $5, statements: $6, end_label: $8}` / `{return_type: $1, scope: $2, name: $3, ports: $5, items: $8, statements: $9, end_label: $11}` | **Every alternative is typed since parser release `1.0.218` (`SV-CORPUS-GRAD.3.44`).** IEEE 1800-2017 A.2.6 gives the production two forms — the NON-ported `[ interface_identifier . \| class_scope ] function_identifier ;` with `tf_item_declaration` items, and the PORTED `… function_identifier ( [ tf_port_list ] ) ;` with `block_item_declaration` items — each with an optional out-of-block prefix, hence four branches. No `kind`: a consumer discriminates the LRM form by KEY PRESENCE — `ports` is present exactly on the ported branches (2, 4; `[]` for `f()`), `scope` exactly on the out-of-block branches (3, 4; the `out_of_block_tf_scope` node — the pair `[<interface_identifier>, {kind: "dot"}]` or the `class_scope` node). `items` holds `tf_item_declaration` nodes (`{kind: "block_item" \| "tf_port", body}`) on the non-ported branches and BARE `block_item_declaration` nodes on the ported ones, exactly as the standard's two productions read; `return_type` is the matched `function_data_type_or_implicit`; `name` a clean identifier node; `end_label` `[]` or the `[{kind: "colon"}, <identifier>]` pair. Drops `semi`, `lparen` / `rparen`, `kw_endfunction`. ⚠️ Through `1.0.217` branches 2–4 arrived as the RAW `Sequence` envelope (a positional array of 10 / 8 / 11 elements) — measured over the corpus at 2 211 of 3 411 bodies, the ported form being the most common function in any corpus; branch 1 is BYTE-IDENTICAL across the release. See [Typed Function and Task Bodies](#typed-function-and-task-bodies-schema-30). |
| `task_declaration_sv_2017` | `-> {lifetime: $2, body: $3}` | Parallel to `function_declaration_sv_2017`. |
| `task_declaration_sv_2023` | `-> {dynamic_override: $2, lifetime: $3, body: $4}` | Parallel to `function_declaration_sv_2023`. |
| `task_body_declaration` (4 branches; **schema 30**) | per-branch typed — `{name: $1, items: $3, statements: $4, end_label: $6}` / `{name: $1, ports: $3, items: $6, statements: $7, end_label: $9}` / `{scope: $1, name: $2, items: $4, statements: $5, end_label: $7}` / `{scope: $1, name: $2, ports: $4, items: $7, statements: $8, end_label: $10}` | The task twin of `function_body_declaration` (IEEE 1800-2017 A.2.7), every alternative typed since `1.0.218`; no `return_type` — a task is void by definition. `ports` present exactly on the ported branches, `scope` exactly on the out-of-block ones; `items` typed vs bare as above. ⚠️ Through `1.0.217` branches 2–4 were the RAW envelope (9 / 7 / 10 elements). The corpus writes no body of branch 3 (an out-of-block, non-ported task) on any lane — its shape is held by the pin `control_tf_body_task_out_of_block_class.sv` and the AST-shape contract sample `task_body_out_of_block_interface`. |
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
| `sequence_expr` (+ 11 `seq_*` cascade rules) | `sequence_expr := seq_or_expr -> $1`; operator layers `-> {kind, lhs, rest}`; `seq_unary` branches `-> {kind: "expression"/"instance"/"paren"/"first_match"/"clocking", …}`; `seq_delay_expr` `-> {kind: "delay_head"/"delay_binary", …}`; `seq_throughout_expr` `-> {kind: "throughout", condition, body}` | **Since 1.0.148 (SV-0010, `H.12.5.8.3.1.1`)** `sequence_expr` is an IEEE 1800 §16 (Table 16-3) **precedence cascade**, replacing the flat directly-left-recursive form that could not parse the infix operators. Loosest→tightest `or > and > intersect > within > throughout > ## > unary`: each operator layer (`seq_or_expr`/`seq_and_expr`/`seq_intersect_expr`/`seq_within_expr`) is `inner inner_tail inner_tail*` emitting `{kind:"or"/"and"/"intersect"/"within", lhs, rest:[first_tail, …more_tails]}`, or its sole-element passthrough `-> $1`; `seq_throughout_expr` is right-assoc `expression_or_dist throughout seq_throughout_expr -> {kind:"throughout", condition, body}`; `seq_delay_expr` carries the `##` cycle-delay forms (`delay_head`/`delay_binary`); `seq_unary` carries the leaf forms (`expression`/`instance`/`paren`/`first_match`/`clocking`). **Carrier compatibility:** a single-operand sequence (`a [*2]`, `first_match(a)`, `x throughout a`) is byte-identical to pre-1.0.148 because each cascade level unwraps via `-> $1`; only the previously-REJECTED infix forms gain the new `{kind, lhs, rest}` shapes (schema stays 6 — strictly-more-permissive). |
| `property_expr` (dispatch + 5 `prop_*` cascade rules per profile) | `property_expr := property_expr_sv_2017 -> {kind:"sv_2017", body} \| property_expr_sv_2023 -> {kind:"sv_2023", body}`; each profile `property_expr_sv_<p> := prop_until_sv_<p> -> $1` over `prop_until > prop_iff > prop_or > prop_and > prop_primary`; infix layers `-> {kind:"until"/"s_until"/"until_with"/"s_until_with"/"iff"/"or"/"and", lhs, rhs}`; `prop_primary` branches the unary/base kinds (`sequence`/`strong`/`weak`/`paren`/`not`/`nexttime*`/`always*`/`eventually*`/`accept_on`-family/`if`/`case`/`imp_minus`/`imp_assign`/`sequence_or_assign`/`instance`/`clocking`) | **Since 1.0.149 (SV-0011, `H.12.5.8.3.2`)** the per-profile `property_expr_sv_2017`/`property_expr_sv_2023` are an IEEE 1800 §16 (Table 16-3) **binary-infix precedence cascade** (the property-layer dual of the 1.0.148 sequence cascade), replacing the flat directly-left-recursive form that could not parse the property infix operators `until`/`s_until`/`until_with`/`s_until_with`/`iff` (and `or`/`and` over property operands). Loosest→tightest `until-family > iff > or > and > primary`, all right-associative inline `head op self` (so e.g. `a until b iff c` → `until(a, iff(b,c))`; `a and b or c` → `or(and(a,b), c)`). **Carrier compatibility:** every previously-VALID property parse is byte-identical — each cascade level unwraps via `-> $1`, the nested `{kind:"sv_2017"/"sv_2023"}` dispatch wrapper is preserved (`prop_primary` bodies still reference `property_expr`), and `prop_primary` is byte-identical; the newly-parsing infix forms reuse the EXISTING `{kind, lhs, rhs}` carriers (schema stays 6 — strictly-more-permissive, no carrier change). **Documented limitation:** the tight prefix ops `not`/`nexttime`/`s_nexttime` (level 6) keep pre-fix dispatch-body precedence, so `not a until b` → `not (a until b)` (unchanged from pre-fix). |
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
| `wait_statement` (3 branches) | per-branch typed | Kind labels: `"wait"` (`{condition, body}`) / `"wait_fork"` (bare) / `"wait_order"` (`{events: [$3, $4::2*], action}` — wait order(e1, e2, ..., eN) action_block; flat array of hierarchical_identifier values per slice 58 audit). |
| `event_trigger` (3 branches) | per-branch `{kind, body}` | Dispatches the active profile's event-trigger rule: `"sv_2017"` / `"sv_2023"` / `"verilog_2005"`. (SV-0023 / schema 16: was 2 branches — the `verilog_2005` body is new.) |
| `event_trigger_sv_2017` (2 branches) | per-branch typed | Kind labels (IEEE 1800-2017 §15.5 / A.6.5): `"blocking"` (`{name}` — `-> name;`, the blocking named-event trigger) / `"non_blocking"` (`{control, name}` — `->> [delay_or_event_control] name;`, the nonblocking trigger; `control` is `[]` for `->> name;`). **SV-0023 / schema 16 corrected the previously SWAPPED labels and added the `->>` token.** |
| `event_trigger_sv_2023` (2 branches) | per-branch typed | Same 2 kinds as sv_2017 plus a trailing `select` field per branch (IEEE 1800-2023 Annex A `nonrange_select` extension) — so `-> e[0];` / `->> e[0];` carry the array index. |
| `event_trigger_verilog_2005` (1 branch) | typed | Kind `"blocking"` (`{name, select}` — IEEE 1364-2005 A.6.5 `-> hei { [ expression ] } ;`; `select` is the `bit_select` array-index chain, `[]` for a plain `-> e;`). No `->>` (SystemVerilog-only). **New at SV-0023 / schema 16** — closes the 1364-2005 event-array-trigger accept-gap. |
| `procedural_timing_control_statement` | `-> {control: $1, body: $2}` | Standard timing-controlled statement form: `<control> statement_or_null`. Reached from `statement_item.kind == "procedural_timing_control"`. |
| `procedural_timing_control` (3 branches) | per-branch `{kind, body}` | Kind labels: `"delay"` (#N delay) / `"event"` (@(...) event) / `"cycle"` (##N cycle delay). |
| `subroutine_call` (5 branches) | per-branch typed | Kind labels: `"class_scoped_tf"` (`{body}`) / `"tf"` (`{body}`) / `"system_tf"` (`{body}` — $display etc.) / `"method"` (`{body}`) / `"randomize"` (`{std_scope, body}` — std_scope is `[]` for plain randomize, `[<kw_std, ::>]` for std::randomize). |
| `subroutine_call_statement` (2 branches) | per-branch typed | Kind labels: `"call"` (`{body}` — wraps subroutine_call with trailing `;`) / `"void_cast"` (`{body}` — `void'(func_call);` discards return value). |
| `seq_block` | `-> {label, declarations, statements, end_label}` | Drops `kw_begin` / `kw_end`. `label` is optional `( colon block_identifier )?` (e.g., `begin : my_block`). |
| `par_block` | `-> {label, declarations, statements, join, end_label}` | Same as seq_block plus `join` field carrying the typed `join_keyword` (discriminates `join` / `join_any` / `join_none` per SV-Slice-7). |
| `case_statement` (3 branches; **schema 31**) | plain `-> {unique_priority, keyword, expr, items: {first, rest}}`; `matches` `-> {kind: "matches", unique_priority, keyword, expr, items: {first, rest}}`; `inside` `-> {kind: "inside", unique_priority, keyword: {kind: "case"}, expr, items: {first, rest}}` | IEEE 1800-2017 A.6.7's three productions. Drops `kw_endcase` and parens. `keyword` is the typed `case_keyword` (case/casez/casex); on the `inside` form — which A.6.7 admits only with the `case` keyword — it is the literal `{kind: "case"}` node. `items.first` / `items.rest` hold `case_item` nodes on the plain form, `case_pattern_item` nodes on `matches`, `case_inside_item` nodes on `inside`. **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the `matches` and `inside` forms are typed**; through `1.0.218` they were the raw nine-element envelope (16 + 15 in the `sv_2017` corpus). The plain form is byte-identical (`kind` absent = plain). `unique_priority` is `[]` or the typed `unique_priority` node. |
| `case_keyword` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"case"` / `"casez"` / `"casex"`. |
| `case_item` (2 branches) | per-branch typed | Kind labels: `"expr_list"` (`{exprs: [$1, $2::2*], body}` — flat array of case_item_expression labels per slice 58 audit) / `"default"` (`{body}` — drops `kw_default` and optional colon). |
| `property_case_item` (2 branches; **schema 31**) | expression arm `-> {expressions: [$1, $2::2*], body: $4}`; default arm `-> {kind: "default", body: $3}` | IEEE 1800-2017 A.2.10, the arms of a `case` property expression (`property_expr`'s `{kind: "case", expr, items}`). **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the default arm is typed** — the shape `case_item` / `case_pattern_item` / `case_inside_item_sv_2017` already give theirs; through `1.0.218` it was the raw four-element envelope. The corpus writes no property case item on any lane; the two `control_property_case_*.sv` pins and the shape-contract sample hold the rule. The expression arm is byte-identical (`kind` absent). |
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
| `production_sv_2017` | `-> {return_type, name, ports, rules: [$5, $6::2*]}` | Drops `colon` and trailing `semi`. `return_type` is the optional `data_type_or_void` prefix. `ports` is the optional `( lparen tf_port_list rparen )?` slot. `rules` is a flat array of `rs_rule` shapes (slice 58 audit dropped the `bitwise_or` separators). |
| `production_item_sv_2017` | `-> {name, args}` | `args` is the optional `( lparen list_of_arguments rparen )?` slot — `[]` for plain reference, `[<arg list>]` when invoking with arguments. |
| `rs_case` | `-> {expr, items: {first, rest}}` | Drops `kw_case` / `kw_endcase` / parens. |
| `rs_case_item_sv_2017` (2 branches) | per-branch typed | Kind labels: `"expr_list"` (`{exprs: [$1, $2::2*], body}` per slice 58 audit) / `"default"` (`{body}`). |
| `rs_case_item_sv_2023` (2 branches) | per-branch typed | Same kinds as sv_2017; uses rs_production_item per LRM 2023 namespacing. |
| `rs_code_block` | `-> {body}` | Exposes the Quantified iteration of `( data_declaration* statement_or_null* )*` directly — each entry is `[data_declaration*-array, statement_or_null*-array]`. |
| `rs_if_else_sv_2017` | `-> {condition, then_body, else_body}` | `else_body` is `[]` for if-without-else, `[<production_item>]` when present. |
| `rs_if_else_sv_2023` | `-> {condition, then_body, else_body}` | Parallel; uses rs_production_item. |
| `rs_prod_sv_2017` (5 branches) | per-branch `{kind, body}` | Kind labels: `"production_item"` / `"code_block"` / `"if_else"` / `"repeat"` / `"case"`. |
| `rs_prod_sv_2023` (5 branches) | per-branch `{kind, body}` | Same 5 kinds as sv_2017; first branch is rs_production_item. |
| `rs_production_sv_2023` | `-> {return_type, name, ports, rules: [$5, $6::2*]}` | Parallel to production_sv_2017 (slice 58 audit). |
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
| `data_type` (15 branches) | per-branch typed | Kind labels: `"integer_vector"` (`{base, signing, dims}` — bit/logic/reg with optional signing + packed dims) / `"integer_atom"` (`{base, signing}` — byte/shortint/int/longint/integer/time) / `"non_integer"` (`{base}` — shortreal/real/realtime) / `"struct_union"` (`{header, packed_signing, members: {first, rest}, dims}` — Category C, kept as-is) / `"enum"` (`{base_type, names: [$4, $5::2*], dims}` — slice 58 audit applied to the enum_name_declaration list) / `"string"` / `"chandle"` / `"event"` (bare; no body) / `"virtual_interface"` (`{interface_keyword, name, params, modport}`) / `"scoped_data_type"` / `"known_unscoped_data_type"` / `"class_type"` / `"provisional_class_type"` / `"covergroup"` / `"type_reference"` (each `{body}`). |
| `data_type_or_implicit` (2 branches) | per-branch `{kind, body}` | Kind labels: `"data_type"` / `"implicit"`. |
| `data_type_or_void` (2 branches) | per-branch typed | Kind labels: `"data_type"` (`{body}`) / `"void"` (bare). |
| `data_type_or_incomplete_class_scoped_type_sv_2023` (2 branches) | per-branch `{kind, body}` | Kind labels: `"data_type"` / `"incomplete_class_scoped"`. |
| `implicit_data_type` | `-> {signing, dims}` | `signing` is `[]` or typed signing slot. `dims` is the packed_dimension* iteration. |
| `unpacked_dimension` / `unpacked_dimension_size_sv_only` (**since `1.0.222`, `SV-CORPUS-GRAD.3.50`**) | `{kind: "range", body: $2}` on the range alternative; the size alternative passes the helper's `{kind: "expression", body: $2}` through (`-> $1`) | An unpacked dimension: `[ a : b ]` on every profile (`{kind: "range"}`); the IEEE 1800 SIZE form `[ N ]` (`{kind: "expression"}`) on `sv_2017` / `sv_2023` alone — the helper `unpacked_dimension_size_sv_only` is `@profiles`-gated, so under `verilog_2005` (IEEE 1364-2005 A.2.5 `dimension` / `range` are `[ a : b ]` alone) the size form REJECTS at the `[`. The node a consumer reads is unchanged by the split: the dispatcher adds no wrapper. |
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
| `list_of_parameter_assignments_sv_2017` (2 branches) | per-branch `{kind, items: [$1, $2::2*]}` (slice 58 audit) | Kind labels: `"ordered"` (positional `#(8, 16)`) / `"named"` (keyword `#(.N(8), .M(16))`). |
| `list_of_parameter_value_assignments_sv_2023` (2 branches) | per-branch `{kind, items: [$1, $2::2*]}` (slice 58 audit) | Same kinds as sv_2017. |
| `named_parameter_assignment` | `-> {name: $2, value: $4}` | Drops `dot` / parens. `value` is `[]` for `.name()`, `[<param_expression>]` for `.name(expr)`. |
| `named_argument` | `-> {name: $2, value: $4}` | Same shape as named_parameter_assignment. |
| `list_of_arguments` (3 branches) | per-branch `{kind, body}` | Kind labels: `"ordered"` / `"named"` / `"mixed"` (positional + trailing named, e.g., `f(1, 2, .x(3))`). |
| `list_of_arguments_ordered` | `-> [$1, $2::2*]` | Flat array `[<optional expression>, <optional expression>, ...]`. Each element is `[]` for empty arg, `[<expression>]` otherwise (slice 58 audit). |
| `list_of_arguments_named` | `-> [$1, $2::2*]` | Flat array of named_argument shapes (slice 58 audit). |
| `list_of_arguments_mixed` | `-> {head: $1, named: [$3, $4::2*]}` | `head` is the typed `list_of_arguments_mixed_head` (positional prefix). `named` is a flat array of trailing named_argument shapes (slice 58 audit). |
| `list_of_arguments_mixed_head` (2 branches) | per-branch typed | Kind labels: `"single"` (`{body}` — single positional argument) / `"chain"` (`{expr, rest}` — recursive: optional expression followed by comma followed by recursive head). |
| `list_of_clocking_decl_assign` / `list_of_defparam_assignments` / `list_of_genvar_identifiers` / `list_of_net_assignments` / `list_of_net_decl_assignments` / `list_of_param_assignments` / `list_of_path_inputs` / `list_of_path_outputs` / `list_of_specparam_assignments` / `list_of_type_assignments` / `list_of_variable_assignments` / `list_of_variable_decl_assignments` (12 rules) | `-> [$1, $2::2*]` | Uniform flat array of inner-rule shapes. Slice 58 audit replaced the prior `{first, rest}` mini-mixed-array. |
| `list_of_interface_identifiers` / `list_of_port_identifiers` / `list_of_variable_identifiers` (3 rules) | `-> [$1, $2::2*]` (clean `[{name, dims}]` record array; **1.0.117 / schema 3**) | Each element is the new annotated record rule `interface_identifier_decl` / `port_identifier_decl` / `variable_identifier_decl` `-> {name, dims}` (identifier + trailing unpacked/variable dimension list per LRM). _(≤ 1.0.116 / schema ≤ 2 history: `-> {first: {name, dims}, rest}` — raw `[[comma, id, [dim…]], …]` `rest` envelope; corrected at POST-SV-AUDIT.2.4b, field names preserved.)_ |
| `list_of_tf_variable_identifiers` | `-> [$1, $2::2*]` (clean `[{name, dims, init}]`; **1.0.117 / schema 3**) | Each element is the new annotated record rule `tf_variable_identifier_decl -> {name, dims, init}`; `init` is `( assign expression )?`. _(≤ 1.0.116 history: `-> {first: {name, dims, init}, rest}`; corrected at POST-SV-AUDIT.2.4b.)_ |
| `list_of_variable_port_identifiers` | `-> [$1, $2::2*]` (clean `[{name, dims, init}]`; **1.0.117 / schema 3**) | Each element is the new annotated record rule `variable_port_identifier_decl -> {name, dims, init}`; `init` is `( assign constant_expression )?`. _(≤ 1.0.116 history: `-> {first: {name, dims, init}, rest}`; corrected at POST-SV-AUDIT.2.4b.)_ |
| `list_of_cross_items` | `-> {first, second, rest}` | Cross requires ≥2 items per LRM A.2.11 — `first` and `second` are required, `rest` is the trailing iteration. |
| `list_of_checker_port_connections` (2 branches) | per-branch `{kind, items: [$1, $2::2*]}` (slice 58 audit) | Kind labels: `"ordered"` / `"named"`. |
| `list_of_port_connections` (2 branches) | per-branch `{kind, items: [$1, $2::2*]}` (slice 58 audit) | Kind labels: `"named"` / `"ordered"`. PEG ordered choice tries named first. |
| `cond_predicate` | `-> [$1, $2::2*]` | LRM A.6.7.1 `&&&`-separated chain of expression-or-cond_pattern values. Flat array per slice 58 audit. |
| `cond_pattern` | `-> {expression, pattern}` | The `expr matches pattern` form used in conditional_statement guards. Drops `kw_matches`. |
| `expression_or_cond_pattern` (2 branches) | per-branch `{kind, body}` | Kind labels: `"expression"` / `"cond_pattern"`. |
| `pattern_sv_2017` (6 branches) | per-branch typed | Kind labels: `"variable_capture"` (`{name}` — `.name` form, binds matched value) / `"wildcard"` (`.*`) / `"expression"` (`{body}`) / `"tagged"` (`{name, sub_pattern}` — tagged-union pattern) / `"ordered"` (`{patterns: [$3, $4::2*]}` — `'{p1, p2}` positional struct pattern; flat array per slice 58 audit) / `"named"` (`{entries: [$3, $4::2*]}` — `'{n1: p1, n2: p2}` keyed pattern; clean `[{name, pattern}]` record array via the new shared annotated record rule `assignment_pattern_entry := member_identifier colon pattern -> {name, pattern}`; **1.0.117 / schema 3**, POST-SV-AUDIT.2.4b, field names preserved. _≤ 1.0.116 history: `{entries: {first: {name, pattern}, rest}}` — raw `[[comma, mid, colon, pattern], …]` `rest` envelope._). |
| `pattern_sv_2023` (7 branches) | per-branch typed | Same 6 kinds as sv_2017 plus `"parenthesized"` (`{body}` — `(pattern)` form added in LRM 2023 A.6.7.1 grammar expansion). |
| `assignment_pattern` (5 branches; **schema 31**) | positional `-> {exprs: [$3, $4::2*]}` (empty `'{}`: `{exprs: []}`); structure-keyed `-> {kind: "structure", keys: [$3, $6::2*], values: [$5, $6::4*]}`; array-keyed `-> {kind: "array", keys: [$3, $6::2*], values: [$5, $6::4*]}`; replication `-> {kind: "replication", count: $3, exprs: [$5, $6::2*]}` | IEEE 1800-2017 A.6.7.1's four productions (+ the empty pattern, SV-0072); drops tick / braces / colons / commas. **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the keyed and replication forms are typed**; through `1.0.218` they were the raw seven- / eight-element envelope (524 + 56 + 60 in the `sv_2017` corpus). `keys[i]` pairs with `values[i]`; a key is the typed `structure_pattern_key` (`{kind: "member" \| "pattern", body}`) or `array_pattern_key` (`{kind: "expression" \| "pattern_key", body}`) node — `'{default: 0}` parses as the structure form (the earlier alternative wins the tie; A.6.7.1 admits `default` in both key productions and only the target type decides). The positional form is byte-identical (`kind` absent). |
| `expression` (2 branches) | per-branch `{kind, body}` | Kind labels: `"base"` (typed expression_base) / `"conditional"` (typed conditional_expression). ⚠️ **Schema 27 (release `1.0.213`, `SV-CORPUS-GRAD.3.41`): the `"inside"` branch is RETIRED** — `x inside { … }` is now a chain element of `expression_base`'s operand chain (see `expression_chain_step` below), never a wrapper around its left operand. |
| `expression_base` (3 branches) | per-branch typed | Kind labels: `"tagged_union"` (`{body}`) / `"operand_chain"` (`{first, rest}` — the operator chain: `rest` is one `expression_chain_step` per operator, in source order; precedence is the consumer's, per IEEE 1800-2017 Table 11-2) / `"paren_op_assign"` (`{body}`). |
| `expression_chain_step` (2 branches, unannotated) | passthrough | One step of the operand chain. A BINARY step passes through as the three-element array `[<binary_operator {kind}>, [<attribute_instance …>], <expression_operand>]` (unchanged since SV-Slice-46); an INSIDE step is the `inside_operator` object. A consumer discriminates by JSON type — array = binary operator with its right operand, object = the set-membership operator with its set — and applies Table 11-2 to the flat sequence, where `inside { … }` is a postfix operator on the relational row (`< <= > >= inside`): `[b, &, a, inside{…}]` groups as `b & (a inside {…})`, `[a, +, b, inside{…}]` as `(a + b) inside {…}`. |
| `expression_operand` (3 branches) | per-branch typed | Kind labels: `"unary"` (`{op, attributes, primary}`) / `"inc_or_dec"` (`{body}`) / `"primary"` (`{body}`). |
| `expression_or_dist` | `-> {expr, dist}` | Wraps an expression with optional `dist { ... }` clause for constraint distribution. |
| `constant_expression` | `-> {first, rest, ternary}` | Binary-op chain with optional `?:` ternary tail. `ternary` is `[]` for non-ternary, `[<? attrs expr : expr>]` for ternary. |
| `constant_expression_operand` (2 branches) | per-branch typed | Kind labels: `"unary"` (`{op, attributes, primary}`) / `"primary"` (`{body}`). |
| `inside_operator` (2 branches) | per-branch `{kind, body}` | Kind labels: `"sv_2017"` / `"sv_2023"` — the profile arm that parsed the set. **Schema 27** (release `1.0.213`): replaces `inside_expression` (`{expr, ranges}`, whose `expr` grouped the WHOLE left chain — `b & a inside {…}` read as `(b & a) inside {…}`, against Table 11-2). The left operand is now the chain to the element's left, so the node carries no `expr`. |
| `inside_operator_sv_2017` / `inside_operator_sv_2023` | `-> {kind: "inside", ranges}` | The `inside { range_list }` set per LRM A.8.3 `inside_expression ::= expression inside { open_range_list }` (2023: `range_list`). sv_2017 `ranges` is the typed `open_range_list`; sv_2023 the typed `range_list`. The `kind: "inside"` discriminator is what the pins' arm claim `operand_chain>inside` reads. |
| `conditional_expression` | `-> {predicate, attributes, then_expr, else_expr}` | Ternary `? :` form. The `&question` positive lookahead is preserved unchanged from source grammar. |
| `tagged_union_expression_sv_2017` / `tagged_union_expression_sv_2023` | `-> {name, value}` | `tagged member_name [value]` form. sv_2023 uses `primary` for value (more restrictive); sv_2017 uses `expression`. |
| `primary_literal` (4 branches) | per-branch `{kind, body}` | Kind labels: `"number"` / `"time_literal"` / `"unbased_unsized_literal"` (`'0` / `'1` / `'x` / `'z`) / `"string_literal"`. |
| `string_literal` / `string_literal_sv_2017` / `string_literal_sv_2023` / `string_literal_v2005` (**since `1.0.221`, `SV-CORPUS-GRAD.3.47`**) | `{kind: "double_quoted", body: $2}` on every double-quoted twin; `{kind: "triple_quoted", body: $2}` on the `sv_2023` twin's `"""…"""` alternative | The string literal, one twin per edition (the SELECTION-dispatcher pattern) because the three standards disagree: IEEE 1800-2017 / 1800-2023 §5.9 admit a `\` + newline CONTINUATION inside a string and refuse a bare newline; IEEE 1364-2005 §3.6 / A.8.8 refuse both; IEEE 1800-2023 §5.9 adds the TRIPLE-QUOTED string (`"""…"""`, any character except `\` inside — a bare `"` and a bare newline included, escapes as `\` + any character). `body` is the SOURCE TEXT verbatim, quotes excluded, continuation included: §5.9 says the backslash and the newline are *ignored*, which is the consumer's reading, not a rewrite of the source. Through `1.0.220` one terminal served every profile and REJECTED the continuation (the regex crate's `.` cannot match a newline) while ADMITTING a raw newline; the `triple_quoted` kind is an addition reachable only on text no earlier release accepted, so schema 32 is unchanged. |
| `binary_operator` (29 branches) | per-branch `{kind}` (bare) | Kind labels: `"plus"` / `"minus"` / `"star"` / `"slash"` / `"percent"` / `"equal"` / `"not_equal"` / `"case_equal"` / `"case_not_equal"` / `"wildcard_equal"` / `"wildcard_not_equal"` / `"logical_and"` / `"logical_or"` / `"power"` / `"less_than"` / `"less_equal"` / `"greater_than"` / `"greater_equal"` / `"bitwise_and"` / `"bitwise_or"` / `"bitwise_xor"` / `"reduction_xnor_alt"` / `"reduction_xnor"` / `"shift_right"` / `"shift_left"` / `"arithmetic_shift_right"` / `"arithmetic_shift_left"` / `"implies"` / `"iff_arrow"`. |
| `unary_operator` (11 branches) | per-branch `{kind}` (bare) | Kind labels: `"plus"` / `"minus"` / `"bang"` / `"tilde"` / `"bitwise_and"` / `"reduction_nand"` / `"bitwise_or"` / `"reduction_nor"` / `"bitwise_xor"` / `"reduction_xnor"` / `"reduction_xnor_alt"`. |
| `primary_sv_2017` (15 branches) | per-branch typed | Kind labels: `"literal"` / `"call"` / `"hierarchical"` (`{scope, name, select}`) / `"empty_array_concat"` / `"multiple_concat"` (`{body, select}`) / `"concat"` / `"let"` / `"paren"` / `"cast"` / `"assign_pattern"` / `"streaming_concat"` / `"sequence_method"` / `"this"` (bare) / `"system_dollar"` (bare) / `"null_class_assign"` (`{local_n, scope}` — rare LRM null:= form). |
| `primary_hier_scope_prefix` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"class_qualifier"` / `"package_scope"`. Helper rule extracted from inline `( kw_class_qualifier \| non_typedef_package_scope )?`. |
| `instance_or_class_scope` (NEW; 2 branches) | per-branch typed | Kind labels: `"instance"` (`{handle}`) / `"class_scope"` (`{body}`). Helper rule extracted from inline `( implicit_class_handle dot \| class_scope )?`. |
| `hierarchical_btf_identifier` (3 branches; schema 28) | per-branch `{kind, …}` | Kind labels: `"tf"` (`{body}`), `"block"` (`{body}`), `"method"` (`{scope, name}` — `scope` is the `class_scope` node). ⚠️ **Schema 28 (release `1.0.215`, `SV-CORPUS-GRAD.3.42`):** IEEE 1800-2017 A.9.3 spells the third alternative `[ hierarchical_identifier . \| class_scope ] method_identifier`; in this grammar's longest-match tournament only the class-scope form is live (`a.m` and a bare `m` tie with alternatives 1 and 2, which win as earlier), so the alternative is spelled `class_scope method_identifier` and `scope` is never absent. Until 1.0.214 the inline group's branch 1 wore alternative 2's template: `@@(begin C::m)` (class `C` declared) carried `scope: {kind: "block", body: <class_scope>}`. |
| `implicit_handle_or_class_scope` (2 branches, unannotated; schema 28) | passthrough | `this .` as the pair `[{kind: "this"}, {kind: "dot"}]`, or a `class_scope` node. Read by `constraint_primary_sv_2017` / `_sv_2023` (`scope`) and the `null_class` receiver alternatives (`prefix`). Until 1.0.214 `solve this.x before this.y;` / `disable soft this.x;` carried `scope: {name: {kind: "dot"}, scope: {kind: "this"}, select: "<invalid_sequence_access>"}` (the 2023 twin also `call: "<invalid_sequence_access>"`). |
| `method_call_receiver_qualifier` (2 branches, unannotated; schema 28) | passthrough | The `class_qualifier` literal (`SV-CORPUS-GRAD.13c.2m`) or a `non_typedef_package_scope` node — `method_call_receiver_sv_2017` / `_sv_2023` alternative 2's `qualifier`. Until 1.0.214 the `pkg::` branch would have worn `{kind: "hierarchical", qualifier, name: "<invalid_sequence_access>"}` whenever that alternative won a final tree. |
| `nonrange_lvalue_scope` (3 branches, unannotated; schema 28) | passthrough | `nonrange_variable_lvalue.scope`: the `this .` pair, a `package_scope` node (`{kind: "package", name}`) or a `class_scope` node. Until 1.0.214 `this.arr = new[4];` carried `scope: {name: {kind: "dot"}, scope: {kind: "this"}, select: "<invalid_sequence_access>"}`. |
| `ps_or_hierarchical_array_scope` (3 branches, unannotated; schema 28) | passthrough | `ps_or_hierarchical_array_identifier.scope` (the `foreach` array): the `this .` pair, a `class_scope` node or a `non_typedef_package_scope` node. Until 1.0.214 `foreach (this.arr[i])` carried `scope: {name: {kind: "dot"}, scope: {kind: "this"}}` — the host's own `{scope, name}` template around the pair. |
| `ps_type_identifier_scope_sv_2017` / `_sv_2023` (3 branches each, unannotated; schema 28) | passthrough | `ps_type_identifier_*.scope`: the `local ::` footnote literal (branch 0, dead on legal text), a `non_typedef_package_scope` node or a `class_scope` node. One helper per edition because the footnote literal differs (43 / 48). |
| `split_callable_receiver_scope` (3 branches, unannotated; schema 28) | passthrough | `split_hierarchical_callable_receiver.scope`: the `class_qualifier` literal, a `non_typedef_package_scope` node or a `class_scope` node. The `pkg::` / `C::` branches were never shadowed (a one-template host shadows branch 0 only) — verible `nested_member_access.sv`'s `nested_class0::handle1::handle2.nested_function()` is byte-identical across the lift. |
| `out_of_block_tf_scope` (2 branches, unannotated; schema 28) | passthrough | `interface_identifier .` as the pair `[<interface_identifier>, {kind: "dot"}]`, or a `class_scope` node — the `scope` field of `function_body_declaration` / `task_body_declaration` branches 3 and 4 (out-of-block definitions) since schema 30, and the raw envelope's scope element from schema 28 to 29. Until 1.0.214 `function void I.h();` put `{return_type: {body: "I"}, name: {kind: "dot"}, items: "<invalid_sequence_access>", statements: "<invalid_sequence_access>", end_label: "<invalid_sequence_access>"}` into that raw envelope; `function void C::g();` is byte-identical across the `1.0.215` lift. |
| `constant_primary_sv_2017` (15 branches) | per-branch typed | Kind labels: `"literal"` / `"ps_parameter"` (`{name, select}`) / `"specparam"` / `"genvar"` / `"formal_port"` / `"enum"` (`{scope, name}`) / `"multiple_concat"` / `"concat"` / `"function_call"` / `"let"` / `"paren"` / `"cast"` / `"assign_pattern"` / `"type_reference"` / `"null"` (bare). |
| `enum_id_scope_prefix` (NEW; 2 branches) | per-branch `{kind, body}` | Kind labels: `"package_scope"` / `"class_scope"`. Helper rule extracted from inline `( non_typedef_package_scope \| class_scope )?` in constant_primary_sv_2017's enum branch. |
| `primary_sv_2023` (15 branches) | per-branch typed | Same 15 kind labels as primary_sv_2017 except `"call"` adds optional `select` field — `{body, select}` (LRM 2023 allows `f()[0]` array-indexed call). Reuses `primary_hier_scope_prefix` and `instance_or_class_scope` helpers. |
| `constant_primary_sv_2023` (16 branches) | per-branch typed | sv_2017's 15 kinds plus `"empty_array_concat"` (`{body}`) added per LRM 2023; also `"function_call"` adds optional `select` field. Reuses `enum_id_scope_prefix` helper. |
| `attr_spec` | `-> {name, value}` | LRM A.9.1 attribute spec: `name [= value]`. `value` is `[]` for bare attribute, `[<assign, expr>]` when explicit. |
| `cast` / `constant_cast` | `-> {type, body}` | LRM cast form `type'(expr)`. Drops tick / parens. |
| `concatenation` / `constant_concatenation` | `-> [$2, $3::2*]` | LRM `{e1, e2, ...}`. Flat array of expression shapes (slice 58 audit). Drops braces. |
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
| `dist_list` | `-> [$1, $2::2*]` | LRM dist clause iteration. Flat array of dist_item shapes (slice 58 audit). |
| `dist_item_sv_2017` | `-> {value, weight}` | Single value-range with optional weight. |
| `dist_item_sv_2023` (2 branches) | per-branch typed | Kind labels: `"value"` (`{value, weight}`) / `"default"` (`{weight}` — `default :/ weight` form per LRM 2023). |
| `dist_weight` (2 branches) | per-branch typed | Kind labels: `"equal"` (`{weight}` — `:=` operator, equal share) / `"proportional"` (`{weight}` — `:/` operator, divided weight). |
| `range_list_sv_2023` / `open_range_list_sv_2017` | `-> [$1, $2::2*]` | Flat array of value_range / open_value_range shapes (slice 58 audit). |
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
| `clocking_declaration` (2 branches; **schema 31**) | `-> {default_keyword, name, event, items, end_label}`; global `-> {kind: "global", name: $3, event: $4, end_label: $7}` | IEEE 1800-2017 A.6.11's two productions; drops `kw_clocking`, `kw_endclocking`, `semi`. **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the `global clocking` form is typed** (no `items` — the production has no clocking items); through `1.0.218` it was the raw seven-element envelope (27 in the `sv_2017` corpus). The first form is byte-identical (`kind` absent). |
| `clocking_direction` (4 branches) | per-branch typed | Kind labels: `"input"` (`{skew}`) / `"output"` (`{skew}`) / `"input_output"` (`{input_skew, output_skew}`) / `"inout"` (bare). |
| `clocking_event_sv_2017` (2 branches; **schema 31**) | `@id` `-> {body: $2}`; `@( event_expression )` `-> {kind: "expression", body: $3}` | IEEE 1800-2017 A.6.11 `clocking_event`. **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the parenthesised form is typed exactly as the `sv_2023` twin types it**; through `1.0.218` it was the raw four-element envelope `[at_sign, lparen, event_expression, rparen]` — 542 of them in the `sv_2017` corpus, every `@(posedge clk)` of every clocking block. The `@id` form is byte-identical (`kind` absent). |
| `clocking_event_sv_2023` (3 branches) | per-branch `{kind, body}` | Kind labels: `"ps"` / `"hierarchical"` / `"expression"`. LRM 2023 expansion. |
| `clocking_item` (3 branches) | per-branch typed | Kind labels: `"default_skew"` (`{skew}`) / `"direction"` (`{direction, decls}`) / `"assertion"` (`{attributes, body}`). |
| `list_of_path_delay_expressions` (5 branches; **schema 31**) | one `-> {body: $1}`; `-> {kind: "two", trise, tfall}`; `-> {kind: "three", trise, tfall, tz}`; `-> {kind: "six", t01, t10, t0z, tz1, t1z, tz0}`; `-> {kind: "twelve", t01, t10, t0z, tz1, t1z, tz0, t0x, tx1, t1x, tx0, txz, tzx}` | IEEE 1364-2005 A.7.4 = 1800-2017 A.7.4 = 1800-2023 A.7.4: one, two, three, six or twelve path delay values (§30.4.1), each position named by the standard; the `delay.body` of a `simple_path_declaration` / `edge_sensitive_path_declaration` under `path_delay_value`'s `{kind: "bare" \| "paren", body}`. Each value is the `{body}` node of its `t*_path_delay_expression` rule. **Since `1.0.219` (`SV-CORPUS-GRAD.3.45`) the four multi-value forms are typed by their LRM names**; through `1.0.218` they were the raw comma-separated envelope (193 + 4 + 5 + 2 on `sv_2017`, 109 + 2 + 3 + 1 on the 1364-2005 lane). Two non-LRM alternatives the grammar had carried since its first promotion — a second six-value spelling that never won the tie, and a twelve-value spelling whose middle four positions were the literal text `tx_path_delay_expression` — are deleted in `1.0.219`. The single-value form is byte-identical (`kind` absent). |
| `clocking_skew` (2 branches) | per-branch typed | Kind labels: `"edge"` (`{edge, delay}`) / `"delay"` (`{body}`). |
| `edge_identifier` (3 branches) | per-branch `{kind}` (bare) | Kind labels: `"posedge"` / `"negedge"` / `"edge"`. |
| `method_prototype` (2 branches) | per-branch `{kind, body}` | Kind labels: `"task"` / `"function"`. |
| `class_constructor_arg_sv_2023` (2 branches) | per-branch typed | Kind labels: `"tf_port_item"` (`{body}`) / `"default"` (bare; LRM 2023 explicit-default arg). |
| `class_constructor_arg_list_sv_2023` | `-> [$1, $2::2*]` | Flat array of class_constructor_arg shapes (slice 58 audit). |
| `class_constructor_declaration_sv_2017` | `-> {class_scope, ports, decls, super_call, statements, end_label}` | Drops `kw_function`, `kw_new`, `semi`, `kw_endfunction`. |
| `class_constructor_declaration_sv_2023` | `-> {class_scope, ports, decls, super_call, statements, end_label}` | Parallel shape; uses `class_constructor_super_args` helper for the inner `( list_of_arguments \| kw_default )?` parens-Or in the super.new clause. |
| `class_constructor_super_args` (NEW; 2 branches) | per-branch typed | Kind labels: `"args"` (`{body}` — `super.new(args);`) / `"default"` (bare — `super.new(default);`). Helper rule extracted from inline parens-Or to dodge task #38. |
| `tf_port_list` | `-> [$1, $2::2*]` | Function/task port list. Flat array of tf_port_item shapes (slice 58 audit — this is the rule the user pointed at to trigger the audit). |
| `tf_port_item` | `-> {attributes, direction, var_keyword, data_type, port_spec}` | Single port-item per LRM A.2.7. |
| `tf_port_direction_sv_2017` (2 branches) | per-branch typed | Kind labels: `"port_direction"` (`{body}`) / `"const_ref"` (bare). |
| `tf_port_direction_sv_2023` (2 branches) | per-branch typed | Kind labels: `"port_direction"` (`{body}`) / `"ref"` (`{const_keyword, static_keyword}` — LRM 2023 expanded ref form). |
| `function_prototype_sv_2017` | `-> {return_type, name, ports}` | Drops `kw_function`. |
| `function_prototype_sv_2023` | `-> {dynamic_override, return_type, name, ports}` | Adds LRM 2023 dynamic_override slot. |
| `task_prototype_sv_2017` / `task_prototype_sv_2023` | `-> {[dynamic_override,] name, ports}` | Parallel to function_prototype. |
| `dpi_function_proto` | `-> {body}` | The DPI import's function prototype. Since `1.0.216` (`SV-CORPUS-GRAD.3.43`) the body is `dpi_function_prototype`, not `function_prototype`: a caller split whose only difference is the return slot (IEEE 1800-2017 §35.5.5 / Annex A footnote 21 — a DPI result is a small value). The typed shape is byte-identical to `function_prototype`'s. |
| `dpi_function_prototype` (2 branches) | per-branch `{kind, body}` | Kind labels `"sv_2017"` / `"sv_2023"`, exactly as `function_prototype`. |
| `dpi_function_prototype_sv_2017` | `-> {return_type, name, ports}` | Mirrors `function_prototype_sv_2017`; `return_type` is a `dpi_function_result_type` node, whose shape is the `data_type_or_void` shape. |
| `dpi_function_prototype_sv_2023` | `-> {dynamic_override, return_type, name, ports}` | Mirrors `function_prototype_sv_2023` (the 2023 override slot on a DPI prototype: A.2.6 `dpi_function_proto ::= function_prototype`). |
| `dpi_function_result_type` (2 branches) | per-branch typed | Kind labels: `"data_type"` (`{body}`, the body a `dpi_result_data_type` node) / `"void"` (bare) — the `data_type_or_void` shape, restricted to the eleven §35.5.5 result types. |
| `dpi_result_data_type` (8 branches) | per-branch typed, each `data_type`'s OWN template | Kind labels `"integer_vector"` (`{base, signing, dims: []}` — scalar `bit` / `logic`, a `[` refused), `"integer_atom"` (`{base, signing}` — `byte` / `shortint` / `int` / `longint`), `"non_integer"` (`{base}` — `real` / `shortreal`), `"string"`, `"chandle"`, `"scoped_data_type"` (`{body: {scope, type, dims: []}}`), `"known_unscoped_data_type"` (`{body: {type, dims: []}}`), `"type_reference"` (`{body}`). A consumer reading a DPI result sees exactly the `data_type` shapes it already knows; `reg`, `integer`, `time`, `realtime`, enum / struct / union literals, `event`, virtual interfaces, class types and any packed dimension are REFUSED at parse time, so they never appear here. |
| `dpi_result_non_integer_type` (2 branches) | pass-through / `{kind: "real"}` | `non_integer_type` without `realtime`. |
| `let_port_item` | `-> {attributes, type, name, dims, init}` | LRM A.2.8 let port. |
| `let_port_list` | `-> [$1, $2::2*]` | Flat array of let_port_item shapes (slice 58 audit). |
| `net_decl_assignment` | `-> {name, dims, init}` | Drops `assign` (init has the body). |
| `variable_decl_assignment` (3 branches) | per-branch typed | Kind labels: `"variable"` (`{name, dims, init}`) / `"dynamic_array"` (`{name, unsized_dim, dims, init}`) / `"class"` (`{name, init}` — `class_var = new` form). |
| `net_lvalue` (3 branches) | per-branch typed | Kind labels: `"name"` (`{name, select}`) / `"concatenation"` (`{items: [$2, $3::2*]}` — flat array per slice 58 audit) / `"pattern"` (`{type, body}`). |
| `variable_lvalue` (4 branches) | per-branch typed | Kind labels: `"name"` (`{scope, name, select}` — scope uses NEW `variable_lvalue_scope` helper) / `"concatenation"` (`{items: [$2, $3::2*]}` — flat array per slice 58 audit) / `"pattern"` (`{type, body}`) / `"streaming_concatenation"` (`{body}`). |
| `variable_lvalue_scope` (NEW; 2 branches) | per-branch typed | Kind labels: `"instance"` (`{handle}` — implicit_class_handle dot prefix) / `"package_scope"` (`{body}`). Helper extracted from inline `( implicit_class_handle dot \| non_typedef_package_scope )?` to dodge task #38. |
| `always_construct` | `-> {keyword, body}` | LRM A.1.4 always-construct (`always`/`always_comb`/`always_latch`/`always_ff` followed by a single `statement`). `keyword` is itself typed via `always_keyword`. `body` is the procedural statement. Referenced from `module_common_item.kind == "always".body`. |
| `always_keyword` (4 branches) | per-branch `{kind: "<name>"}` | Kind labels: `"always"` / `"always_comb"` / `"always_latch"` / `"always_ff"`. Bare typed object — drops the keyword token (redundant with `kind`). |
| `import_export` (2 branches) | per-branch `{kind: "<name>"}` | Kind labels: `"import"` / `"export"`. Bare typed — used by `package_import_declaration` and `modport_tf_ports_declaration`. |
| `modport_simple_ports_declaration` | `-> {direction, port}` | LRM A.2.9 modport simple-port declaration. `direction` is the typed `port_direction`. `port` is the typed `modport_simple_port`. |
| `modport_clocking_declaration` | `-> {name}` | Drops `kw_clocking`; surfaces only the clocking_identifier. |
| `modport_declaration` | `-> {items: [$2, $3::2*]}` | Drops `kw_modport`/`semi`. `items` is a flat array of `modport_item` shapes (Category A — slice 58 pattern). |
| `modport_item` | `-> {name, ports: [$3, $4::2*]}` | Drops `lparen`/`rparen`. `name` is the modport_identifier; `ports` is a flat array of `modport_ports_declaration` shapes. |
| `modport_ports_declaration` (3 branches) | per-branch `{kind, attributes, body}` | Kind labels: `"simple"` (modport_simple_ports_declaration) / `"tf"` (modport_tf_ports_declaration) / `"clocking"` (modport_clocking_declaration). `attributes` carries the leading `attribute_instance*`. |
| `modport_simple_port` (2 branches) | per-branch typed | Kind labels: `"identifier"` (`{name}` — bare port_identifier reference) / `"explicit"` (`{name, expression}` — `.name(expr)` synthesized port form; `expression` is `[]` if absent). |
| `modport_tf_port` (2 branches) | per-branch typed | Kind labels: `"method_prototype"` (`{body}`) / `"tf_identifier"` (`{name}`). |
| `modport_tf_ports_declaration` | `-> {import_export, ports: [$2, $3::2*]}` | `import_export` is typed (`{kind: "import"}` or `{kind: "export"}`). `ports` is a flat array of `modport_tf_port` shapes. |

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

## The `dist` Constraint Operator

*Parser release `1.0.179` (`SV-CORPUS-GRAD.3.18`, ledger `SV-0049`). **Schema `19` → `20`.***

IEEE 1800-2017 A.2.10 defines the distribution operand as:

```
expression_or_dist ::= expression [ dist { dist_list } ]
dist_list          ::= dist_item { , dist_item }
dist_item          ::= value_range [ dist_weight ]
dist_weight        ::= := expression | :/ expression
```

⛔ **The `{ }` on the first line are literal SystemVerilog braces; the `{ }` on the second line are
BNF repetition.** Two lines apart, same characters, opposite meanings. Before `1.0.179` the grammar
read the first pair as repetition too, and the consequences were not limited to a rejection.

| you write | before `1.0.179` | from `1.0.179` |
|---|---|---|
| `x dist {100 := 1, 200 := 2}` | **rejected** at the `:=` | parses |
| `x dist { [100:102] :/ 1, 200 := 2}` | **rejected** at the `[` | parses |
| `soft x dist {5, 8};` | parses — but as **one** item whose value is the concatenation `{5, 8}` | parses as the **two** items the LRM defines |
| `x dist 100 := 1;` (no braces) | **accepted** — no such production exists | correctly rejected |

The third row is the one to read twice. It parsed, so no pass/fail check anywhere reported it; the
only visible symptom was in the emitted tree, where the dist item's value carried
`"kind": "concat"` instead of the individual values.

### Migration

For any input that already parsed, the `dist` slot grew from two elements to four:

```jsonc
// before 1.0.179 — `soft x dist {5, 8};`
"dist": [ ["…","dist"], [ [ /* ONE item, value = concat of 5 and 8 */ ] ] ]

// from 1.0.179
"dist": [ ["…","dist"], {"kind":"lbrace"}, [ /* item 5 */, /* item 8 */ ], {"kind":"rbrace"} ]
```

- A consumer that indexed `dist[1]` to reach the list must now read `dist[2]`.
- A consumer that read a single item whose value was a concatenation must now read the individual
  `dist_item`s.

The pre-`1.0.179` shape was a bug, not a stable schema — it could not represent a weighted
distribution at all.

### Bound worth knowing

`dist_weight` is modelled as the two tokens `:` `=` (and `:` `/`) rather than a fused `:=` / `:/`,
so interior white space — `x dist {100 : = 1}` — is currently tolerated where the LRM's operator
spelling arguably forbids it. That is a **strictness** question in the same family as the
`time_literal` white-space item, tracked separately in `docs/tasks/SV-CORPUS-GRAD.md`; it is not
introduced by this release and no corpus file depends on it.

## The Config `use` Clause and Parameter Overrides

*Parser release `1.0.180` (`SV-CORPUS-GRAD.3.19`, ledger `SV-0050`). Schema stays `20`.*

A `config` block selects which cell binds to which instance, and it can override parameters while
doing so. Before `1.0.180` the parameter-override spelling that IEEE 1800's own examples use did
not parse at all:

```systemverilog
config cfg;
  design rtlLib.top;
  instance top     use #(.WIDTH(32));      // rejected before 1.0.180
  instance top.a1  use #(.W(top.WIDTH));   // rejected before 1.0.180
  instance top.a1  use #();                // rejected before 1.0.180
endconfig
```

### Why it was missing — the standard disagrees with itself

This is worth spelling out, because it is *not* the failure mode behind the other recent grammar
fixes in this book. Those (`SV-0044` cycle-delay ranges, `SV-0049` the `dist` operator) were cases
where the LRM wrote a literal delimiter and PGEN's grammar read it as BNF metasyntax. Here PGEN's
transcription was **right**, and the standard is internally inconsistent:

- **Annex A / Syntax 33-4** defines `use_clause` with three alternatives and **no `#` anywhere**:
  `use [lib.]cell [:config]`, `use .P(1), …`, `use [lib.]cell .P(1), …`.
- **Clause 33.4.3** then writes every one of its parameter-override examples as `use #( … )` —
  seven times, and identically in both IEEE 1800-2017 and IEEE 1800-2023.

PGEN follows the union of the two, on three grounds that are internal to the standard rather than a
matter of taste. Annex A's preamble subordinates itself — *"The normative text description contained
within the clauses and annexes of this standard provide additional details on the syntax"*. The
BNF-only reading would make clause 33.4.3's explicit *"Configurations may not use positional
parameter notation to override parameters"* meaningless, since positional notation cannot even be
written under the brace-less alternatives. And every parameter-override `use` clause in PGEN's
vendored external corpus — all 12 of them — uses the `#( … )` spelling.

### What still rejects, on purpose

```systemverilog
instance top.a1 use #(32);            // REJECTED — positional override
instance top.a1 use #(32, .W(8));     // REJECTED — mixed positional
```

Clause 33.4.3 forbids positional parameter notation in a configuration, so the `#( … )` list accepts
`named_parameter_assignment` only. This is why the grammar does **not** reuse
`parameter_value_assignment` — that rule is exactly `# ( [ list_of_parameter_assignments ] )`, and
`list_of_parameter_assignments` admits ordered assignments. An ordinary module instantiation is
unaffected and still accepts both notations:

```systemverilog
adder #(8, 16)  a1();   // fine — ordinary instantiation, positional allowed
adder #(.W(8))  a2();   // fine
```

### What a consumer sees

The `use` clause is carried as a raw token sequence inside its `config_rule_statement`, whose `kind`
discriminator (`inst_use` / `cell_use` / `inst_liblist` / `cell_liblist` / `default_liblist`) tells
you which shape to expect:

```json
{"kind": "inst_use",
 "clause": {"name": {"head": {"body": "top"}, "scope_chain": []}},
 "body": [[[], "use"],
          {"kind": "hash"}, {"kind": "lparen"},
          [{"name": {"body": "WIDTH"}, "value": {"…": "32"}}, []],
          {"kind": "rparen"}, []]}
```

The `{"kind": "hash"}` / `{"kind": "lparen"}` / `{"kind": "rparen"}` markers are how you tell the
braced form from the Annex-A brace-less one (`use .WIDTH(32)`), which carries the named assignments
with no surrounding punctuation nodes.

**Schema is unchanged at `20`.** This construct was 100 % unparseable before `1.0.180`, so no shape
a consumer had ever seen could move — verified by dumping the AST of every already-parsing form of
the clause on both parsers and confirming all ten are byte-identical.

### Which profiles accept the overrides

*Parser release `1.0.181` (`SV-CORPUS-GRAD.3.20`, ledger `SV-0051`). Schema stays `20`.*

**Parameter override in a config `use` clause is SystemVerilog-only.** IEEE 1364-2005 declares
exactly one `use_clause` alternative — `use [library_identifier.]cell_identifier[:config]` (Annex A
A.1.5, restated in 13.3) — with no parameter override in any spelling, and the string `use #(` occurs
nowhere in the 2005 text. Since `1.0.181` the parser matches that:

| spelling | `sv_2017` / `sv_2023` | `verilog_2005` |
|---|---|---|
| `use adder;` · `use rtlLib.adder;` · `use rtlLib.adder:config;` | accept | **accept** |
| `use .W(8);` · `use .W(8), .D(16);` | accept | **reject** |
| `use adder .W(8);` · `use rtlLib.adder .W(8);` | accept | **reject** |
| `use #(.W(8));` · `use #();` | accept | **reject** |
| `use #(32);` (positional — 33.4.3 forbids it) | reject | reject |

⛔ **This restricts the config `use` clause only.** An ordinary module instantiation's parameter
value assignment — `adder #(8, 16) a1();`, `adder #(.W(8)) a2();` — is legal Verilog-2001/2005 and
still accepts under every profile. The two surfaces look alike and are governed by different
productions.

**Mechanically**, the four override alternatives live in a `@profiles: ["sv_2017", "sv_2023"]`
sibling rule, `use_clause_param_override_sv_only`, referenced as `use_clause`'s first alternative.
`@profiles` is a rule-level directive, so gating a subset of a rule's alternatives always takes this
shape; the house pattern is `always_keyword` / `always_keyword_sv_only`.

**No consumer-visible shape change.** Under `sv_2017`/`sv_2023` the AST is byte-identical across the
split — a bare rule-reference alternative carrying no return annotation adds no wrapper node — so
existing `inst_use` / `cell_use` readers are unaffected. Under `verilog_2005` the affected spellings
no longer parse at all, so there is no shape to read. If a tool was relying on the old behaviour, it
was requesting `verilog_2005` while parsing SystemVerilog; request `sv_2017` or `sv_2023` instead.

## Directives Inside Module and Class Bodies

*Parser release `1.0.178` (`SV-CORPUS-GRAD.3.14b`, ledger `SV-0048`). Schema stays `19`.*

Before `1.0.178` a compiler directive was tolerated **only at file top level**. The rule
`compiler_directive` was an alternative of `source_text_item` and of nothing else, so the
identical text one line lower rejected:

```systemverilog
`timescale 1ns / 1ps        // parsed
module m;
endmodule
```

```systemverilog
module m;
  `timescale 1ns / 1ps      // REJECTED before 1.0.178
endmodule
```

A legally-placed directive now parses at **module-body** and **class-body item positions**,
and appears in the item array as an ordinary typed item:

```systemverilog
module m;
  `timescale 1ns / 1ps
  wire w;
endmodule
```

```json
"items": [
  { "kind": "compiler_directive", "body": "`timescale 1ns / 1ps" },
  { "kind": "module_or_generate", "body": { "...": "the wire declaration" } }
]
```

The class-body form is identical in shape:

```systemverilog
class c;
  `undef EVIL_MACRO
  int x;
endclass
```

```json
"items": [
  { "kind": "compiler_directive", "body": "`undef EVIL_MACRO" },
  { "kind": "sv_2017", "body": { "...": "the int property" } }
]
```

⚠️ **Consumer impact.** `non_port_module_item` and `class_item` can now yield the kind
`compiler_directive`, which previously appeared only under `source_text`. A consumer that
matches those item kinds exhaustively must add the arm. The node itself is byte-identical to
the top-level one — same `kind`, same string `body` — so no schema change is involved; only
the container is new. ⚠️ The string bodies shown above are the `1.0.178`–`1.0.216` shape;
since `1.0.217` (schema 29) the body is the typed node described in
[Typed Compiler Directives](#typed-compiler-directives-schema-29), at both containers alike.

## Typed Compiler Directives (schema 29)

*Parser release `1.0.217` (`SV-CORPUS-GRAD.3.39`, ledger `SV-0090`). Schema `28 → 29`.*

Through `1.0.216` every tolerated directive — at file top level and at the in-scope item positions —
arrived as **one opaque string**: `{"kind": "compiler_directive", "body": "`timescale 1ns / 1ps"}`. That
carried two defects the corpus measured. A consumer received a line it had to re-lex itself, and the
parser accepted any arguments at all: `` `timescale frump ``, `` `unconnected_drive pull2 ``,
`` `line 1 "f" 3 `` and a bare `` `pragma `` all parsed (verilator `t_timescale_parse_bad.v` /
`t_unconnected_bad.v`, sv-tests `22.12--line-illegal-*.sv`, sv2v `test/error/line_*.sv` — 26 corpus
files on the `sv_2017` lane, every one a form the clause's syntax box refuses).

IEEE 1800-2017 clause 22 gives thirteen directives a syntax box — each captioned *"not in Annex A"*,
which is why no Annex-A-driven slice transcribed them — and `1.0.217` transcribes each box as a
production. The body is now a typed node whose `kind` is the directive's name:

```systemverilog
`timescale 1 ns / 1 ps
`default_nettype none
`line 3 "orig.v" 2
`pragma protect encoding = (enctype = "BASE64", bytes = 128)
`begin_keywords "1364-2005"
`include "x.svh"
```

```json
[
  { "kind": "compiler_directive", "body": { "kind": "timescale",
      "unit":      { "value": "1", "unit": { "kind": "ns" } },
      "precision": { "value": "1", "unit": { "kind": "ps" } } } },
  { "kind": "compiler_directive", "body": { "kind": "default_nettype", "value": { "kind": "none" } } },
  { "kind": "compiler_directive", "body": { "kind": "line", "number": "3",
      "filename": { "kind": "double_quoted", "body": "\"orig.v\"" }, "level": "2" } },
  { "kind": "compiler_directive", "body": { "kind": "pragma", "name": "protect", "expressions": [
      { "kind": "assignment", "keyword": "encoding", "value": { "kind": "group", "items": [
          { "kind": "assignment", "keyword": "enctype",
            "value": { "kind": "string", "body": { "kind": "double_quoted", "body": "\"BASE64\"" } } },
          { "kind": "assignment", "keyword": "bytes",
            "value": { "kind": "number", "body": { "kind": "integral", "body": "128" } } } ] } } ] } },
  { "kind": "compiler_directive", "body": { "kind": "begin_keywords",
      "version": { "kind": "double_quoted", "body": "\"1364-2005\"" } } },
  { "kind": "compiler_directive", "body": { "kind": "opaque", "text": "`include \"x.svh\"" } }
]
```

| `body.kind` | fields | clause |
|---|---|---|
| `resetall`, `celldefine`, `endcelldefine`, `nounconnected_drive`, `end_keywords`, `undefineall` | — | §22.3, §22.10, §22.9, §22.14, §22.5.3 |
| `undef` | `name` (identifier node) | §22.5.2 |
| `timescale` | `unit`, `precision` (`{value, unit: {kind}}`) | §22.7 |
| `default_nettype` | `value` (`{kind}`, eleven values) | §22.8 |
| `unconnected_drive` | `value` (`{kind: "pull0" \| "pull1"}`) | §22.9 |
| `pragma` | `name`, `expressions` (list; `keyword` / `assignment` / `value`; values `group` / `number` / `string` / `identifier`) | §22.11 |
| `line` | `number`, `filename` (string node), `level` | §22.12 |
| `begin_keywords` | `version` (string node) | §22.14 |
| `opaque` | `text` (the whole line, the `1.0.216` string) | — the residue |

**Migration.** `body` was a string; it is an object. Switch on `body.kind`. The residue —
`` `include ``, `` `define ``, `` `ifdef `` / `` `else `` / `` `elsif `` / `` `endif `` / `` `ifndef ``,
a text-macro use, a vendor directive such as `` `default_decay_time `` — keeps the string under
`body.text` with `body.kind == "opaque"`, and is still tolerated only at file top level (the
`.3.14b` in-scope whitelist has no opaque branch). Files whose directives all had legal arguments
change shape and nothing else; the accept set narrows exactly on the illegal forms (the pins
`stimuli/sv/adjudication_repros/invalid_directive_*.sv`, and the corpus rows named in
`docs/tasks/artifacts/sv_corpus_grad/directive_census/census.md`).

**Three readings a consumer should know.** (1) A newline between directive arguments is §5.3 white
space — no box states a line rule, so `` `timescale 1ns `` ⏎ `` / 1ps `` parses (it did not on
`1.0.216`, whose opaque token ended at the newline); only `` `line `` is bound to its line, as §22.12
says ("only white space may appear on the same line … comments are not allowed"), and a bare-name
`` `pragma `` is bounded to its line so it cannot swallow the next item's first identifier. (2) The
magnitude of a `` `timescale `` argument is syntax (`1`, `10`, `100` — §22.7's only definition of it),
where Annex A's own `timeunit 2ps;` stays derivable; the precision-ordering rule is a value rule and
is not judged. (3) `` `begin_keywords `` admits exactly the edition's list: `"1800-2023"` parses under
`sv_2023` and rejects under `sv_2017` and `verilog_2005`; `"1800-2017"` rejects under `verilog_2005`.

### Which directives are tolerated — and which are still rejected

The set is a **whitelist derived from IEEE 1800-2017 clause 22 and IEEE 1364-2005 clause 19
read in full**, not blanket tolerance of any backtick line. The two standards agree name for
name; only `` `undefineall `` is SystemVerilog-only.

| tolerated in-scope | clause | why |
|---|---|---|
| `` `celldefine `` `` `endcelldefine `` | §22.10 / §19.1 | "may appear anywhere in the source description" |
| `` `undef `` | §22.5.2 / §19.3.2 | no placement restriction stated |
| `` `undefineall `` | §22.5.3 (SV only) | "may appear anywhere in the source description" |
| `` `timescale `` | §22.7 / §19.8 | no placement restriction stated |
| `` `pragma `` | §22.11 / §19.10 | no placement restriction stated |
| `` `line `` | §22.12 / §19.7 | "can be specified anywhere within the … source description" |

Everything else still rejects inside a design element, deliberately, and that half is as much
part of the contract as the acceptances:

| still rejected in-scope | clause | why |
|---|---|---|
| `` `resetall `` | §22.3 / §19.6 | "It shall be illegal … within a design element" |
| `` `default_nettype `` | §22.8 / §19.2 | "can be used only outside design elements" |
| `` `unconnected_drive `` `` `nounconnected_drive `` | §22.9 / §19.9 | "shall be specified outside the design element declarations" |
| `` `begin_keywords `` `` `end_keywords `` | §22.14 / §19.11 | "can only be specified outside a design element" |
| `` `include `` `` `define `` `` `ifdef `` `` `ifndef `` `` `else `` `` `elsif `` `` `endif `` | §22.4–§22.6 / §19.3–§19.5 | these **hide or rewrite** the text that follows, so the file is a preprocessing dependency rather than complete parser input |
| `` `__FILE__ `` `` `__LINE__ `` and any user macro | §22.13 | these are predefined/user **text macros**, not directives — see below |

```systemverilog
module m;
  `default_nettype none     // still REJECTED — LRM permits it only outside design elements
endmodule

module m;
  `MY_MACRO(x)              // still REJECTED — an unexpanded macro use, not a directive
endmodule
```

⭐ **Why the macro case matters.** `` `MY_MACRO(x) `` at an item position expands to real
module items. A blanket "tolerate any backtick line" rule would swallow the line, silently
drop everything the macro expands to, and report a **pass** for a file the parser never
actually understood. Rejecting it keeps an unexpanded file honestly distinguishable from a
complete one. Directive names are matched with a trailing word boundary, so
`` `timescale_like_macro `` and `` `undef_not_a_directive `` are macros, not directives.

### Bounds worth knowing

- Tolerance is at **item positions** — between items, never between two tokens of one
  statement. `always @(posedge clk) `timescale …` does not parse.
- Hosts are **module bodies and class bodies**. Generate, interface, program, package and
  checker bodies still reject a legally-placed directive
  (`docs/tasks/SV-CORPUS-GRAD.md` `.3.14d`).
- `` `pragma `` is tolerated as a **line**; its structured §22.11 content is not parsed and its
  effect is not modelled. A `` `pragma protect `` encrypted envelope (§34) still rejects,
  correctly — it stops at the base64 payload, not at the directive.
- Top-level behaviour is unchanged: directives above/below a design element parsed before and
  parse identically now.

## Typed Function and Task Bodies (schema 30)

*Parser release `1.0.218` (`SV-CORPUS-GRAD.3.44`, ledger `SV-0091`). Schema `29 → 30`.*

IEEE 1800-2017 A.2.6 / A.2.7 give a function or task body exactly two forms, each with an optional
out-of-block prefix:

```
function_body_declaration ::=
    function_data_type_or_implicit [ interface_identifier . | class_scope ] function_identifier ;
        { tf_item_declaration } { function_statement_or_null } endfunction [ : function_identifier ]
  | function_data_type_or_implicit [ interface_identifier . | class_scope ] function_identifier ( [ tf_port_list ] ) ;
        { block_item_declaration } { function_statement_or_null } endfunction [ : function_identifier ]
task_body_declaration ::= (the same two forms, without the return type)
```

The grammar spells that as four alternatives — the prefix × the port list — and through `1.0.217` only
the first (`type f; … endfunction`, its ports declared INSIDE the body) carried a return template. The
other three arrived as the raw `Sequence` envelope: a positional array the consumer had to index by
hand, with the port list at `[3]` on a ported function and at `[4]` on an out-of-block ported one. The
corpus census on the `1.0.217` probe (`AST-FIELD-CENSUS:` with the new `--select-keys lifetime,body`
selector + `TF-BODY-CLASSIFY:`) read **2 211 of the 3 411 function / task bodies the parser wrote on
`sv_2017` as that raw envelope** — 1 646 ported functions, 513 ported tasks, 30 out-of-block functions,
22 out-of-block tasks — in 1 060 of the 1 958 accepting files that declare a function or task at all. The
ported form is the most common function in any corpus, and the typed AST could not read it.

Since `1.0.218` every alternative is typed, mirroring alternative 1's field set:

```systemverilog
function int f;  input int a;  return a;  endfunction                 // alt 1 (unchanged)
function int f(int a);  return a;  endfunction : f                   // alt 2 — ports present
function void I.h;  input int a;  endfunction                        // alt 3 — scope present
function int C::g(int a, int b = 2);  return a * b;  endfunction    // alt 4 — scope and ports present
```

```json
{"return_type": {"kind": "data_type", "body": {…}}, "name": {"body": "f"}, "items": [{"kind": "tf_port", "body": {…}}], "statements": […], "end_label": []}
{"return_type": {…}, "name": {"body": "f"}, "ports": [{…}], "items": [], "statements": […], "end_label": [{"kind": "colon"}, {"body": "f"}]}
{"return_type": {…}, "scope": [{"body": "I"}, {"kind": "dot"}], "name": {"body": "h"}, "items": [{"kind": "tf_port", "body": {…}}], "statements": [], "end_label": []}
{"return_type": {…}, "scope": {"body": {"head": {"body": "C"}, "params": [], "scope_chain": []}}, "name": {"body": "g"}, "ports": [{…}, {…}], "items": [], "statements": […], "end_label": []}
```

**How a consumer tells the forms apart.** There is no `kind`: the discriminator is key PRESENCE, which
is the standard's own distinction. `ports` is present exactly on the ported alternatives (2 and 4) — it is
the typed `tf_port_list` array, `[]` for `f()` — and `scope` exactly on the out-of-block ones (3 and 4),
carrying the `out_of_block_tf_scope` node: the pair `[<interface_identifier>, {kind: "dot"}]` for an
interface method (§25.7) or the `class_scope` node for a class method (§8.24). The two forms also differ
in `items`, faithfully: a NON-ported body's items are `tf_item_declaration` nodes, `{kind: "block_item" |
"tf_port", body}`, because that is where its ports live; a ported body's items are bare
`block_item_declaration` nodes, because A.2.6's second production admits no port declaration there. A
consumer that switched on the array length before now reads `"ports" in body` and `"scope" in body`.

**Migration.** A consumer of schema 29 that indexed the raw envelope — `body[0]` for the return type,
`body[3]` / `body[4]` for the port list, `body[6]` / `body[7]` / `body[8]` for the items — reads the
named fields instead; the elements are the same typed nodes in the same order, with the `lparen` /
`rparen` / `semi` / `endfunction` tokens dropped as they are on alternative 1. Alternative 1 itself is
byte-identical (measured: the two in-block pins and every all-typed corpus file). A REPLACEMENT of a raw
envelope by a typed node is the `SV-0014`…`SV-0020` corruption-fix precedent, hence the bump.

**Measured over the corpus.** The corpus AST identity sweep (the 1.0.217 probe vs this release, TOOLBOX §1.5e) reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=8696 ast_moved=1060 verdict_moved=0 both_nonaccepting=6580` and `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=8716 ast_moved=1064 verdict_moved=0 both_nonaccepting=6556`, and over the `verilog_2005` lane `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2270 ast_moved=40 verdict_moved=0 both_nonaccepting=296` — two-sided against the census's prediction (`sv_2017`: predicted 1060, moved 1060, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 1064, moved 1064, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 40, moved 40, predicted-but-identical 0, moved-but-not-predicted 0). No verdict moves on any lane (a return template changes
what the parser emits, never what it accepts): `sv_2017` `transitions: NONE` (pass 9756 / fail 6580 of 16336 rows, 0 timeouts); `sv_2023` `transitions: NONE` (pass 9780 / fail 6556 of 16336 rows, 0 timeouts); `verilog_2005` `transitions: NONE` (pass 2310 / fail 296 of 2606 rows, 0 timeouts). Eight pins — one per alternative of each
rule, the two in-block ones as the controls that cannot move (`control_tf_body_*.sv`, arms `tf_port` /
`!tf_port` pinning WHERE the ports live) — and six AST-shape contract samples, one per newly typed
alternative, parsed AS the body rule.

**Profiles.** The ported alternatives are legal IEEE 1364-2005 too (A.2.6 `function_port_list`, A.2.7
`task_port_list`), so the `verilog_2005` profile's typed AST moves on the same forms; the out-of-block
alternatives are SV-only (interfaces and classes) and reject there as before.

## Typed Partial Alternatives (schema 31)

> **Schema 32 addendum (`1.0.220`, `SV-CORPUS-GRAD.3.46`).** The one `block_data` body this section left without a `kind` — alternative 1, the
> block-scoped integer VECTOR `reg [7:0] v;` / `logic [3:0] t;`, typed by SV-Slice-96 as `{base, signing, dims}` — now carries `data_type`'s own
> `{kind: "integer_vector", base, signing, dims}`. A REPLACEMENT: a reader that recognised the vector form by the ABSENCE of `kind` reads
> `kind == "integer_vector"`; every other alternative is byte-identical. Pre-registered by this section's own census (712 nodes in 259 accepting
> files per SV lane, 130 in 73 on the 1364-2005 lane) and measured two-sided by the corpus AST identity sweep: every predicted file moved, no
> other accepted tree moved a byte, no verdict moved. From schema 32 every alternative a consumer can meet on the `block_data` body carries a
> `kind` or is a passed-through child — the body is discriminated by `kind` exactly as `data_type` is.

*Parser release `1.0.219` (`SV-CORPUS-GRAD.3.45`, ledger `SV-0092`). Schema `30 → 31`.*

`1.0.218` closed with the class measured: the partial-typing census (TOOLBOX §1.5g) read **38 rules typed on some
alternatives and not others, 19 of the untyped alternatives raw envelopes**, in eight rules. A raw envelope is the
positional `Sequence` array a consumer indexes by hand; every one of the nineteen was a consumer-facing shape. The
corpus census on the `1.0.218` probe (`RAW-ENVELOPE-CENSUS:`, TOOLBOX §1.5h, every lane row) read **2 319 of them in
1 088 accepting files on `sv_2017`** (1 779 / 819 on `sv_2023`, 217 / 120 on the 1364-2005 lane):

| rule | alternative | raw on `sv_2017` | typed since `1.0.219` |
|---|---|---:|---|
| `block_data_type` | integer atom in a block (`integer i;`, `int unsigned x;`) | 873 | `{kind: "integer_atom", base, signing}` (= `data_type`) |
| `clocking_event_sv_2017` | `@( event_expression )` | 542 | `{kind: "expression", body}` (= the `sv_2023` twin) |
| `assignment_pattern` | structure-keyed `'{a: 1}` | 524 | `{kind: "structure", keys, values}` |
| `list_of_path_delay_expressions` | two values | 193 | `{kind: "two", trise, tfall}` |
| `assignment_pattern` | replication `'{2{0, 1}}` | 60 | `{kind: "replication", count, exprs}` |
| `assignment_pattern` | array-keyed `'{0: 1}` | 56 | `{kind: "array", keys, values}` |
| `clocking_declaration` | `global clocking` | 27 | `{kind: "global", name, event, end_label}` |
| `case_statement` | `case … matches` | 16 | `{kind: "matches", unique_priority, keyword, expr, items}` |
| `case_statement` | `case … inside` | 15 | `{kind: "inside", unique_priority, keyword: {kind: "case"}, expr, items}` |
| `list_of_path_delay_expressions` | three / six / twelve values | 4 / 5 / 2 | `{kind: "three" \| "six" \| "twelve", …}` by LRM position name |
| `block_data_type` | struct/union, enum, virtual interface in a block | 1 / 0 / 1 | `data_type`'s own `{kind: "struct_union" \| "enum" \| "virtual_interface", …}` |
| `property_case_item` | `default:` arm | 0 | `{kind: "default", body}` |
| `net_type_declaration_sv_2017` | the alias form `nettype wTsum nettypeid2;` | 0 — **shadowed** | `{kind: "nettype", scope, base, name}` — the data-type alternative now refuses an alias head by lookahead, the alias base is store-gated |

**One rule for every new template.** The typed sibling of each rule keeps its shipped shape byte-identically; every
newly typed alternative carries a `kind`, so **`kind` absent is the shipped form**. Where a twin already had a `kind`
vocabulary (`data_type` for the block types, `clocking_event_sv_2023` for the clocking event, the three case-item
rules for the default arm, `nettype_declaration_sv_2023` for the alias form) the same words are used, so a consumer
that reads the twin reads the new node unchanged.

```systemverilog
initial begin integer i; int unsigned x; end     // block_data_type: two integer_atom nodes
s = '{a: 1, b: 2};   arr = '{0: 1, default: 0};   arr = '{2{0, 1}};
clocking cb @(posedge clk); endclocking          // clocking_event_sv_2017: expression
global clocking gcb @(posedge clk); endclocking : gcb
case (u) matches tagged Valid .n: ; default: ; endcase
priority case (a) inside [0:1]: ; default: ; endcase
(a => b) = (1, 2, 3);                            // list_of_path_delay_expressions: three
nettype wTsum nettypeid2;                        // the alias form, IEEE 1800-2017 §6.6.7
```

```json
{"kind": "integer_atom", "base": {"kind": "int"}, "signing": {"kind": "unsigned"}}
{"kind": "structure", "keys": [{"kind": "member", "body": {"body": "a"}}, {"kind": "member", "body": {"body": "b"}}], "values": [{…}, {…}]}
{"kind": "array", "keys": [{"kind": "expression", "body": {…}}, {"kind": "pattern_key", "body": {"kind": "default"}}], "values": [{…}, {…}]}
{"kind": "replication", "count": {…}, "exprs": [{…}, {…}]}
{"kind": "expression", "body": [{"kind": "expression", "edge": {"kind": "posedge"}, "expr": {…}, "iff": []}]}
{"kind": "global", "name": {"body": "gcb"}, "event": {"kind": "sv_2017", "body": {"kind": "expression", "body": […]}}, "end_label": [{"kind": "colon"}, {"body": "gcb"}]}
{"kind": "inside", "unique_priority": {"kind": "priority"}, "keyword": {"kind": "case"}, "expr": {…}, "items": {"first": {"kind": "sv_2017", "body": {"kind": "range_list", …}}, "rest": [{…}]}}
{"kind": "three", "trise": {"body": {…}}, "tfall": {"body": {…}}, "tz": {"body": {…}}}
{"kind": "nettype", "scope": [], "base": {"body": "wTsum"}, "name": {"body": "nettypeid2"}}
```

**Migration.** A consumer that indexed a raw envelope reads the named fields; the elements are the same typed nodes in
the same order, with the punctuation and keyword tokens dropped. Concretely: `pattern[2]` / `pattern[4]` and each
`[comma, key, colon, expr]` iteration → `keys[i]` / `values[i]`; `body[0]` / `body[1]` of a block integer atom → `base`
/ `signing`; the case envelope's `[3]` expression and `[6]` / `[7]` items → `expr`, `items.first`, `items.rest`; the
clocking envelope's `[2]` / `[3]` / `[6]` → `name` / `event` / `end_label`; the event envelope's `[2]` → `body`; the
comma-separated delay list → the LRM position names; the property default arm's `[2]` → `body`.

**The nettype alias form was shadowed, not raw.** IEEE 1800-2017 §6.6.7 gives a nettype declaration a second form,
another name for an existing nettype: `nettype wTsum nettypeid2;`. On the LRM's own example both alternatives of
`net_type_declaration_sv_2017` succeed — a declared nettype is a `type_name` fact, so the data-type form reads `wTsum` as a
`known_unscoped_data_type` — and the longest-match tie went to declaration order: the alias alternative NEVER won, on
either SV profile (the fully typed `sv_2023` twin included), and the AST called the base nettype a data type. Since
`1.0.219` the data-type alternative REFUSES an alias head by negative lookahead — `!( nettype_scope_prefix?
checked_nettype_identifier declaration_identifier semi )`, i.e. `[ scope ] <declared nettype> <identifier> ;` — so the alias alternative
wins those declarations while the order stays as shipped; and the alias base `net_type_identifier`, a bare
`declaration_identifier` through `1.0.218` (one ungated twin per edition), is `checked_nettype_identifier |
wildcard_escape_nettype_identifier` — a declared nettype, or an undeclared name under a wildcard import, exactly how
`net_declaration` reads a net of a user-defined nettype (SV-PARSE-STRICT.2; SV-0037 routed the same ungated names in
`net_port_type`) — the three ungated twins deleted. Two spellings were refuted first: reordering the ungated alternative
read `nettype T wT;` (T a struct typedef, ispras `06.07.02_01.sv`) as an alias — the two-sided corpus sweep caught it as
the one moved-but-not-predicted file — and reordering the gated one broke the certificate's own nettype prelude: the
generator renders an unplanned declaration with the first of two equally cheap alternatives, so `checked_nettype_identifier`'s
witness prelude became `nettype \foo \foo;`, an alias with an undeclared base, and the union certificate read
`UNKNOWN=2` where `1.0.218` had 0. Order is a generation contract as much as a parsing one. The alias alternative now wins
exactly the alias declarations; the corpus carries two per SV lane (verilator `t_nettype.v`, ispras `06.06.07_01.sv`), pre-registered by the census's
`alias_shadowed` note and moved by the release. ⚠️ **One narrowing**: `nettype undeclared_t x;` with no wildcard import in
scope was accepted through `1.0.218` down the ungated alias alternative and is LRM-illegal (§6.6.7: the base must be an
existing nettype); it now rejects on both SV profiles — the census read 0 accepting corpus files down that alternative,
and `invalid_nettype_alias_undeclared_base.sv` pins it.

**Two non-LRM alternatives removed.** `list_of_path_delay_expressions` had carried seven alternatives since the grammar's
first promotion where IEEE 1364-2005, 1800-2017 and 1800-2023 all give five: a second six-value spelling that matched
the same text as the LRM's six-value form and lost the longest-match tie to it on every input (Protocol D: six values
`selected branch 4/7`), and a twelve-value spelling whose middle four positions were the regex terminal
`/tx_path_delay_expression\b/` — the production NAME extracted as literal source text, matched by no file in the
corpus. Neither could produce a tree; both are gone, with their orphaned keyword terminal — with the three ungated
nettype identifier twins, `1641 → 1637` rules.

**Measured over the corpus.** The corpus AST identity sweep (the `1.0.218` probe vs this release, every lane row,
TOOLBOX §1.5e) and the lanes' verdicts are recorded in the integration contract's `1.0.219` note and the census
artifact `docs/tasks/artifacts/sv_corpus_grad/partial_typing_residue/census.md`; the partial-typing census reads
`RAW_ENVELOPE=0` on the released grammar (31 partial rules remain, all of them passthrough alternatives typed one level
down by design).

