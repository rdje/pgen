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
