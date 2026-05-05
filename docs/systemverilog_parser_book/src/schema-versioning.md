# Schema Versioning

This chapter explains how the SystemVerilog parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape.

## Two versioning axes

The systemverilog parser carries **two** version numbers:

1. **Parser release version** — e.g. `1.0.0`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **Schema version** — e.g. `1`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two version numbers move independently.

The contract document `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for both numbers per release.

## What "shape change" means

Any of these triggers a schema version bump:

- A new return annotation lands on a previously-unannotated rule.
- An existing return annotation is restructured.
- A grammar rule changes shape in a way that's user-visible (new branch added, branch removed, sub-rule renamed in a way that affects shape).
- The default fall-through behavior of unannotated rules changes.

These do NOT trigger a schema bump:

- Pure performance optimizations that produce the same AST.
- Internal codegen reorganization that doesn't reach the output.
- Parser-side bug fixes that produce the same shape consumers were already relying on.

The slice campaign for SV will produce many small schema bumps as rules are annotated one-by-one. Each slice gets its own contract-version row.

## Byte-equivalence guarantee

For any input the parser accepts, the AST dump is **byte-deterministic** for a given parser-release version: object keys in canonical order, canonical number formatting, no embedded timestamps or hashes. Re-running the parse on the same input produces an identical JSON value.

This determinism is a **hard guarantee** of the schema. Any non-determinism is a bug — please report via `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.

## Schema version timeline

| Schema version | First parser release | Notable changes |
|---|---|---|
| 0.1.0 | 1.0.0 | **Foundation baseline.** Initial mdbook + integration contract Highlights structure landed. Grammar (`grammars/systemverilog.ebnf`) is un-annotated except for one commented-out trial annotation at line 200. AST dump is the recursive-envelope shape across all rules. Manifest (`systemverilog_v1.json`) carries one stub sample (`minimal_module`) calibrated against the placeholder `current_content_kind: "sequence"`. First post-foundation slice will run the parser, observe the actual content kind, and either confirm or update the manifest. |
| 0.19.0 | 1.0.18 | **SV-Slice-18 batch: UDP truth-table entries typed.** 3 rules: combinational_entry ({inputs, output}), sequential_entry ({inputs, current_state, next_state}), udp_initial_statement ({name, init_val}). Annotation inventory: 122 entries (was 119). |
| 0.18.0 | 1.0.17 | **SV-Slice-17 batch: UDP body sub-tree typed.** 6 rules / 8 annotations: udp_body (2 kinds), udp_input_declaration, udp_output_declaration (2 kinds: wire/reg), combinational_body, sequential_body, list_of_udp_port_identifiers. UDP declaration internals fully typed end-to-end. Annotation inventory: 119 entries (was 111). |
| 0.17.0 | 1.0.16 | **SV-Slice-16 batch: port + port_direction + package_import family typed.** 4 rules / 9 annotations: port (2 kinds: expression/named), port_direction (4 kinds: input/output/inout/ref), package_import_declaration ({items:{first,rest}}), package_import_item (2 kinds: explicit/wildcard). DEFERRED: ansi_port_declaration — task #38 blocker (parens-grouped-Or in branch 0 causes branch-attribution renumbering). Annotation inventory: 111 entries (was 102). |
| 0.16.0 | 1.0.15 | **SV-Slice-15 batch: port-list family + small structural rules typed.** 6 rules / 7 annotations: list_of_ports ({first, rest}), list_of_port_declarations ($2), udp_port_list ({output, inputs: {first, rest}}), udp_declaration_port_list (parallel), anonymous_program ({items}), package_export_declaration (2 kinds: wildcard / explicit). Crossing 100 annotations on the SV grammar — campaign mid-flight. Annotation inventory: 102 entries (was 95). |
| 0.15.0 | 1.0.14 | **SV-Slice-14 batch: bind sub-tree completion + interface_class_declaration + config_declaration.** 5 rules: bind_target_scope (2 kinds), bind_target_instance, bind_target_instance_list ({first, rest}), interface_class_declaration, config_declaration. Bind sub-tree fully typed end-to-end (combined with SV-Slice-13). Annotation inventory: 95 entries (was 89). |
| 0.14.0 | 1.0.13 | **SV-Slice-13 batch: bind_directive + bind_instantiation + package_item per-branch typed.** 3 Or rules: bind_directive (2 kinds: scoped/single), bind_instantiation (4 kinds: program/module/interface/checker), package_item (4 kinds: declaration/anonymous_program/export/timeunits). Consumer gains clean kind dispatch on description's package_item / bind_directive branches. Annotation inventory: 89 entries (was 79). |
| 0.13.0 | 1.0.12 | **SV-Slice-12 batch: UDP declaration family typed.** 4 rules: udp_ansi_declaration, udp_nonansi_declaration, udp_declaration_sv_2017 (5 per-branch kinds: nonansi/ansi/extern_nonansi/extern_ansi/wildcard), udp_declaration_sv_2023 (mirror with positional shift). Special: nonansi branch's `udp_port_declaration udp_port_declaration*` mini-mixed-array uses `port_decls: {first, rest}` workaround. Annotation inventory: 79 entries (was 67). |
| 0.12.0 | 1.0.11 | **SV-Slice-11 batch: program-header sub-tree typed.** 2 rules: program_ansi_header, program_nonansi_header. Same 6-field shape as module/interface headers (attributes, lifetime, name, imports, parameters, ports). Verified on `program p; endprogram\n`: header.name = "p". Annotation inventory: 67 entries (was 65). |
| 0.11.0 | 1.0.10 | **SV-Slice-10 batch: class + package + program declarations typed.** 5 rules: class_declaration_sv_2017 (single sequence with `lifetime:`), class_declaration_sv_2023 (single sequence with `final_specifier:` per LRM-2023), package_declaration (single sequence), program_declaration_sv_2017 (5 per-branch kinds), program_declaration_sv_2023 (mirror with positional shift). Verified on `program p; endprogram\n`. Open follow-up: bare-package input parses incorrectly despite annotation registering — investigation pending. Annotation inventory: 65 entries (was 53). |
| 0.10.0 | 1.0.9 | **SV-Slice-9 batch: interface declarations typed (mirror of module pattern).** 4 rules: interface_ansi_header, interface_nonansi_header, interface_declaration_sv_2017 (5 per-branch kinds: ansi/nonansi/wildcard/extern_nonansi/extern_ansi), interface_declaration_sv_2023. No keyword field on interface headers (only one keyword); otherwise mirrors module pattern. Verified empirically on `interface bus; endinterface\n`: header.name = "bus" (clean string from slice 8). Annotation inventory: 53 entries (was 41). |
| 0.9.0 | 1.0.8 | **SV-Slice-8 batch: identifier-leaf rules typed — clean strings propagate through every identifier field.** 4 leaf rules typed: `simple_identifier`, `escaped_identifier`, `non_keyword_identifier`, `simple_identifier_no_scope`. All use `-> $2` transparent passthrough (drop trivia / lookahead, surface regex-captured name). Massive propagation effect: every typed rule referencing `*_identifier` (module/class/package/interface/etc.) now exposes name fields as clean JSON strings. For `module m; endmodule\n`: header.name was `[[], [[], "m"]]` → now `"m"`. Annotation inventory: 41 entries (was 37). |
| 0.8.0 | 1.0.7 | **SV-Slice-7 batch: 4 multi-rule typing pass — module-header sub-tree.** (a) `module_keyword` per-branch typed (`{kind: "module"}` / `{kind: "macromodule"}`). (b) `lifetime` per-branch typed (`{kind: "static"}` / `{kind: "automatic"}`). (c) `module_ansi_header` typed `{attributes, keyword, lifetime, name, imports, parameters, ports}`. (d) `module_nonansi_header` same field names (only ports source rule differs). Four layers of typed dispatch now end-to-end: source_text_item.kind → description.kind → module_declaration_sv_<profile>.kind → module_<form>_header.keyword.kind. Annotation inventory: 37 entries (was 31). |
| 0.7.0 | 1.0.6 | **SV-Slice-6 batch: 3 multi-rule typing pass.** (a) `attribute_instance` typed `{first: $2, rest: $3}` (drops attr_open/close delimiters). (b) `module_declaration_sv_2017` per-branch typed (5 kind labels: ansi/nonansi/wildcard/extern_nonansi/extern_ansi) with structured fields (header/timeunits/items/end_label, plus attributes/keyword/lifetime/name on wildcard). (c) `module_declaration_sv_2023` same per-branch typing (kind set identical; only the wildcard branch's positional indices shift due to `dot star` vs `dot_star`). Three layers of typed dispatch now end-to-end: source_text_item.kind → description.kind → module_declaration_sv_<profile>.kind. Annotation inventory: 31 entries (was 20). NOTE: `comment_only_source_region` typing attempted in this batch but DEFERRED — blocked by task #38 (parens-grouped-Or trailing-annotation attribution bug). |
| 0.6.0 | 1.0.5 | **SV-Slice-5: `compiler_directive` transparent passthrough.** Annotated `-> $2` to drop the leading `trivia` slot and emit just the matched directive text as a clean JSON string. When `source_text_item.kind == "compiler_directive"`, the body is now a directly-usable string. Heterogeneous body shape per kind: string for compiler_directive, typed object for description. Annotation inventory: 20 entries (was 19). |
| 0.5.0 | 1.0.4 | **SV-Slice-4: `description` per-branch typed.** 8 per-branch annotations on `description`. Each emits `{kind: "<name>", body: $1}` for single-element branches (module/udp/interface/program/package/config_declaration); `{kind: "<name>", attributes: $1, body: $2}` for the two multi-element branches (`attribute_instance* package_item` / `attribute_instance* bind_directive`). Two layers of typed dispatch now end-to-end: source_text_item.kind → description.kind. Annotation inventory: 19 entries (was 11). |
| 0.4.0 | 1.0.3 | **SV-Slice-3: `source_text_item` per-branch typed.** 8 per-branch annotations on `source_text_item`. Every Or branch emits `{kind: "<name>", body: $1}` (or `{kind: "semi"}` for the bodyless `semi` branch). Consumer dispatch on `item["kind"]` instead of structural recursion. Trailing `semi` dropped in `local_parameter_declaration semi` / `parameter_declaration semi` branches. Annotation inventory: 11 entries (was 3). |
| 0.3.0 | 1.0.2 | **SV-Slice-2: `source_text` flatten-spread.** `source_text := source_text_item*` annotated `-> [$1**]`. The `source_text` field of `systemverilog_file` is now a flat array of `source_text_item` shapes — pre-fix it carried the raw Quantified envelope wrapping the iteration. Same accept set, same `type`/`source_text` keys at root level. Annotation inventory: 3 entries (was 2). |
| 0.2.0 | 1.0.1 | **SV-Slice-1: `systemverilog_file` typed.** Dangling `-> {type: "systemverilog_file", source_text: $2}` annotation at line 195 of `grammars/systemverilog.ebnf` was rescued by moving it onto its target rule `systemverilog_file := trivia source_text trivia` (line 184). Removed misleading `//` prefix from `systemverilog_parseable_file`'s annotation (PGEN's EBNF dialect uses `#` for comments, not `//`). Both annotations now register via the documented path. AST root for `module m; endmodule\n` changed from recursive-envelope `Sequence` to typed `{type: "systemverilog_file", source_text: ...}` JSON object. First effective annotation on the systemverilog parser. Manifest `current_content_kind` calibrated to `"json_object"`. |

(The numbers above match the contract document at the time this book was written. The contract is authoritative for the current state — consult it for the live version.)

## Future major version

A schema 1.0.0 milestone will land when the SV annotation campaign completes — that is, when every rule in `grammars/systemverilog.ebnf` carries either a return annotation or a deliberate decision to remain raw envelope. At that point all shape definitions move to Tier 2 (locked) and no further default fall-through changes are expected.

The campaign is in early phase; reaching schema 1.0.0 is on the order of months of slice work.
