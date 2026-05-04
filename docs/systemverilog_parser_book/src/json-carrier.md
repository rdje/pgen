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
