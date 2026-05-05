# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.0.0 / Contract 1.0.0** (foundation baseline).

### 1.0.18 / Contract 1.0.18 — SV-Slice-18 batch: UDP truth-table entries typed

**What changed:** 3 rules / 3 annotations completing the UDP truth-table walk path.

```ebnf
combinational_entry   -> {inputs, output}
sequential_entry      -> {inputs, current_state, next_state}
udp_initial_statement -> {name, init_val}
```

**Annotation inventory:** 122 entries (was 119). +3.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.18 / Contract 1.0.18 Highlights".

### 1.0.17 / Contract 1.0.17 — SV-Slice-17 batch: UDP body sub-tree typed

**What changed:** 6 rules / 8 annotations completing UDP declaration internals.

```ebnf
udp_body                     -> 2 kinds: combinational/sequential
udp_input_declaration        -> {attributes, identifiers}
udp_output_declaration       -> 2 kinds: wire/reg
combinational_body           -> {entries: {first, rest}}
sequential_body              -> {initial, entries: {first, rest}}
list_of_udp_port_identifiers -> {first, rest}
```

**UDP declaration internals fully typed end-to-end** — combined with prior UDP top-level rules (SV-Slice-12) and port lists (SV-Slice-15), consumers walking a `primitive ... endprimitive` get clean typed access at every level.

**Annotation inventory:** 119 entries (was 111). +8 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.17 / Contract 1.0.17 Highlights" with full UDP walker recipe.

### 1.0.16 / Contract 1.0.16 — SV-Slice-16 batch: port + port_direction + package_import family typed

**What changed:** 4 rules / 9 annotations.

```ebnf
port                       -> 2 kinds: expression / named (dot-form)
port_direction             -> 4 kinds: input / output / inout / ref
package_import_declaration -> {items: {first, rest}}
package_import_item        -> 2 kinds: explicit / wildcard
```

**DEFERRED:** `ansi_port_declaration` per-branch typing — branch 0 starts with a parens-grouped Or `( net_port_header | interface_port_header )?` which triggers task #38's branch-attribution bug. Tracked as follow-up either via task #38 fix or grammar refactor.

**Annotation inventory:** 111 entries (was 102). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.16 / Contract 1.0.16 Highlights".

### 1.0.15 / Contract 1.0.15 — SV-Slice-15 batch: port-list family + small structural rules typed

**What changed:** 6 rules / 7 annotations. Every `header.ports` field on every typed module/interface/program/UDP declaration now surfaces a typed shape.

```ebnf
list_of_ports             -> {first, rest}     (mini-mixed-array)
list_of_port_declarations -> $2 (transparent passthrough of inner optional)
udp_port_list             -> {output, inputs: {first, rest}}
udp_declaration_port_list -> {output, inputs: {first, rest}}
anonymous_program         -> {items}
package_export_declaration -> 2 kinds (wildcard / explicit with {first, rest})
```

**Annotation inventory:** 102 entries (was 95). +7 in this batch. **Crossing 100 annotations** — campaign mid-flight.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.15 / Contract 1.0.15 Highlights" with full annotation source + per-rule notes.

### 1.0.14 / Contract 1.0.14 — SV-Slice-14 batch: bind sub-tree completion + interface_class_declaration + config_declaration

**What changed:** 5 rules typed in one batch:

```ebnf
bind_target_scope          -> 2 kinds: module/interface ({kind, name})
bind_target_instance       -> {name, bit_select}
bind_target_instance_list  -> {first, rest} (mini-mixed-array workaround)
interface_class_declaration -> {name, parameters, extends, items, end_label}
config_declaration         -> {name, local_params, design, rules, end_label}
```

**Bind sub-tree fully typed** — combined with SV-Slice-13's bind_directive/bind_instantiation typing, consumers walking a bind directive get clean typed access at every level (target_scope, target_instance, instances, instantiation).

**Annotation inventory:** 95 entries (was 89). +6 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.14 / Contract 1.0.14 Highlights" with full annotation source + bind walker recipe.

### 1.0.13 / Contract 1.0.13 — SV-Slice-13 batch: bind_directive + bind_instantiation + package_item per-branch typed

**What changed:** 3 Or rules typed. Consumers gain clean kind dispatch on description's `package_item` and `bind_directive` branches.

```ebnf
bind_directive       -> 2 kinds: scoped/single
bind_instantiation   -> 4 kinds: program/module/interface/checker
package_item         -> 4 kinds: declaration/anonymous_program/export/timeunits
```

**Annotation inventory:** 89 entries (was 79). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.13 / Contract 1.0.13 Highlights" with full annotation source + consumer dispatch recipe.

### 1.0.12 / Contract 1.0.12 — SV-Slice-12 batch: UDP declaration family typed

**What changed:** 4 rules typed across the UDP (User-Defined Primitive) declaration family — sibling pattern to module/interface/program with one twist: `udp_declaration_sv_*` nonansi branch has a `udp_port_declaration udp_port_declaration*` mini-mixed-array, handled with the `{first, rest}` workaround.

```ebnf
udp_ansi_declaration     -> {attributes, name, ports}
udp_nonansi_declaration  -> {attributes, name, ports}
udp_declaration_sv_2017  -> 5 per-branch kinds: nonansi/ansi/extern_nonansi/extern_ansi/wildcard
udp_declaration_sv_2023  -> mirror of sv_2017 with positional shift in wildcard
```

**Mini-mixed-array workaround:** the nonansi branch's `udp_port_declaration udp_port_declaration*` pattern uses `port_decls: {first: $2, rest: $3}` to surface the required-first + repeat shape. Same idiom as `attribute_instance: {first, rest}` from SV-Slice-6. Consumers iterate `port_decls.first` once then walk `port_decls.rest`.

**Annotation inventory:** 79 entries (was 67). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.12 / Contract 1.0.12 Highlights" — full annotation source + consumer dispatch recipe + mini-mixed-array workaround documentation.

### 1.0.11 / Contract 1.0.11 — SV-Slice-11 batch: program-header sub-tree typed (sibling of module/interface headers)

**What changed:** 2 rules typed: `program_ansi_header`, `program_nonansi_header`. Both expose the same 6-field shape: `attributes`, `lifetime`, `name`, `imports`, `parameters`, `ports`. Same field names as module / interface header pairs (program is sans `keyword:` since it only has one keyword).

**Verified on `program p; endprogram\n`:** `header.name = "p"` (clean string from SV-Slice-8), all 6 fields present.

**Annotation inventory:** 67 entries (was 65). +2 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.11 / Contract 1.0.11 Highlights".

### 1.0.10 / Contract 1.0.10 — SV-Slice-10 batch: class + package + program declarations typed

**What changed:** 5 rules typed: `class_declaration_sv_2017` and `class_declaration_sv_2023` (single-sequence shapes; sv_2017 has `lifetime:`, sv_2023 has `final_specifier:` per LRM-2023 semantics), `package_declaration` (single sequence with attribute_instance* prefix), `program_declaration_sv_2017` and `program_declaration_sv_2023` (5 per-branch kinds each, mirroring module/interface).

**Verified empirically on `program p; endprogram\n`:**

```text
source_text[0]: {kind: "description", body: {
    kind: "program_declaration",
    body: {kind: "ansi", header: {...}, timeunits: [], items: [], end_label: []}
}}
```

**Module/interface/program tests still pass** with the same regenerated parser — annotations didn't introduce regressions.

**Open follow-up:** `package p; endpackage\n` parse rejected at position 0 despite `package_declaration` being in `description`'s Or set. Annotation registered correctly per the inventory; runtime parse failure appears pre-existing. Module/interface/program parsing unaffected. Tracking separately.

**Class top-level parse:** `class C; endclass\n` is also rejected — but this is expected, since class_declaration isn't directly in source_text_item's reachable set; class declarations are reached through `package_item` or other subsidiary contexts.

**Annotation inventory:** 65 entries (was 53). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.10 / Contract 1.0.10 Highlights".

### 1.0.9 / Contract 1.0.9 — SV-Slice-9 batch: interface declarations typed (full mirror of module pattern)

**What changed:** 4 rules typed: `interface_ansi_header`, `interface_nonansi_header`, `interface_declaration_sv_2017` (5 per-branch kinds), `interface_declaration_sv_2023` (same 5 kinds with positional shift). Interface declarations now have the same typed surface as module declarations. 4-layer typed dispatch end-to-end + clean identifier strings.

**Empirical for `interface bus; endinterface\n`:**

```text
source_text[0]: {kind: "description", body: {
    kind: "interface_declaration",
    body: {
        kind: "ansi",
        header: {name: "bus", attributes: [], lifetime: [], imports: [], parameters: [], ports: []},
        timeunits: [], items: [], end_label: []
    }
}}
```

**Difference from module pattern:** No `keyword:` field on interface_<form>_header (only one `interface` keyword exists). Otherwise field names mirror `module_<form>_header`.

**Annotation inventory:** 53 entries (was 41). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.9 / Contract 1.0.9 Highlights".

### 1.0.8 / Contract 1.0.8 — SV-Slice-8 batch: identifier-leaf rules typed (clean strings propagate through every identifier field)

**What changed:** 4 identifier-leaf rules typed with `-> $2` transparent passthrough. Highest-leverage slice yet — every rule that resolves to an identifier now surfaces a clean JSON string instead of the raw envelope chain.

```ebnf
simple_identifier          := trivia /[a-zA-Z_][a-zA-Z0-9_$]*/                            -> $2
escaped_identifier         := trivia /\\[!-~]+/                                            -> $2
non_keyword_identifier     := !reserved_non_keyword_identifier identifier                  -> $2
simple_identifier_no_scope := trivia /[a-zA-Z_][a-zA-Z0-9_$]*(?![ \t\r\n]*::)/             -> $2
```

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — header.name was raw envelope chain:
"header": {"keyword": {"kind": "module"}, "name": [[], [[], "m"]], ...}

# Post — clean string:
"header": {"keyword": {"kind": "module"}, "name": "m", ...}
```

**Propagation:** `simple_identifier` / `escaped_identifier` are leaves of `identifier` (transparent Or). `non_keyword_identifier` strips the negative lookahead. `declaration_identifier` / `module_identifier` / `class_identifier` / `package_identifier` / `interface_identifier` etc. are all transparent aliases — they automatically surface clean strings now. Every future-typed rule that exposes an identifier as a named field gets a clean string for free.

**Annotation inventory:** 41 entries (was 37). +4 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.8 / Contract 1.0.8 Highlights".

### 1.0.7 / Contract 1.0.7 — SV-Slice-7 batch: `module_keyword` + `lifetime` + `module_ansi_header` + `module_nonansi_header` typed (4 layers of dispatch end-to-end)

**What changed:** 4-rule batch slice typing the header sub-tree of module declarations. Four layers of typed dispatch are now end-to-end.

```ebnf
module_keyword         := kw_module      -> {kind: "module"}
                        | kw_macromodule -> {kind: "macromodule"}

lifetime               := kw_static      -> {kind: "static"}
                        | kw_automatic   -> {kind: "automatic"}

module_ansi_header     := attribute_instance* module_keyword (lifetime)? module_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                       -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

module_nonansi_header  := attribute_instance* module_keyword (lifetime)? module_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                       -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

**Empirical for `module m; endmodule\n`:** the previously-raw `header:` field of the ansi-kind module_declaration_sv_2017 now resolves to a typed object with `keyword: {kind: "module"}` and named fields for all 7 components. ANSI and non-ANSI forms expose the same field names — consumer code walking the header is uniform across both.

**Annotation inventory:** 37 entries (was 31). +6 in this batch (2 module_keyword + 2 lifetime + 1 module_ansi_header + 1 module_nonansi_header).

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.7 / Contract 1.0.7 Highlights" — has the per-rule annotation source code + 4-layer consumer dispatch recipe.

### 1.0.6 / Contract 1.0.6 — SV-Slice-6 batch: `attribute_instance` + `module_declaration_sv_2017/2023` typed (3 layers of dispatch end-to-end)

**What changed:** Multi-rule batch slice. Three rules typed in one pass: `attribute_instance` (`{first, rest}` shape), `module_declaration_sv_2017` (5 per-branch kind labels: ansi/nonansi/wildcard/extern_nonansi/extern_ansi), `module_declaration_sv_2023` (same kind labels as sv_2017; wildcard branch's positional indices shift to accommodate `dot star` vs `dot_star`).

**Three layers of typed dispatch end-to-end** — `source_text_item.kind` (SV-Slice-3) → `description.kind` (SV-Slice-4) → `module_declaration_sv_<profile>.kind` (this slice). For `module m; endmodule\n`:

```json
{
  "type": "systemverilog_file",
  "source_text": [
    {
      "kind": "description",
      "body": {
        "kind": "module_declaration",
        "body": {
          "kind": "ansi",
          "header": [<module_ansi_header envelope>],
          "timeunits": [],
          "items": [],
          "end_label": []
        }
      }
    }
  ]
}
```

**Annotation inventory:** 31 entries (was 20). +11 in this batch.

**`comment_only_source_region` typing was attempted in this batch but DEFERRED** — blocked by task #38 (parens-grouped-Or trailing-annotation attribution bug). The rule's two `( a | b )` parens-grouped Or expressions cause the trailing `-> ...` annotation to fail to register on the rule. Annotation reverted; this rule's typing is gated on task #38's resolution OR a grammar refactor that flattens the parens-grouped Ors into named helper rules.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.6 / Contract 1.0.6 Highlights" — has the per-rule annotation source code + consumer dispatch recipe.

### 1.0.5 / Contract 1.0.5 — SV-Slice-5: `compiler_directive` transparent passthrough (clean directive text)

**What changed:** `compiler_directive := trivia /` `` `[^\r\n]*/`` `` `(line 226 of `grammars/systemverilog.ebnf`) annotated with `-> $2`. Drops the leading `trivia` slot and emits just the matched directive text as a clean JSON string. Consumer code receives a directly-usable string for `source_text_item.body` when `source_text_item.kind == "compiler_directive"`.

**Empirical pre/post for an input with `` `define FOO bar `` + `module m; endmodule\n`:**

```text
# Pre — body was raw envelope of `trivia regex_capture`:
"source_text": [
  {"kind": "compiler_directive", "body": [<trivia envelope>, "`define FOO bar"]}
]

# Post — body is the clean directive string:
"source_text": [
  {"kind": "compiler_directive", "body": "`define FOO bar"}
]
```

**Annotation inventory:** 20 entries (was 19). New: `compiler_directive`.

**Heterogeneous body shape per `source_text_item.kind`** — when kind is `"description"`, body is a typed object; when kind is `"compiler_directive"`, body is a string. Same pattern regex AST uses for typed atoms.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.5 / Contract 1.0.5 Highlights".

### 1.0.4 / Contract 1.0.4 — SV-Slice-4: `description` per-branch typed (`kind:` discriminator on 8 branches; `attribute_instance*` preserved)

**What changed:** 8 per-branch annotations on `description` (line 957 of `grammars/systemverilog.ebnf`). Every Or branch now emits a typed object with a `kind:` discriminator. The two multi-element branches (`attribute_instance* package_item` / `attribute_instance* bind_directive`) preserve the leading attribute_instance* prefix as a separate `attributes:` field while keeping the inner construct as `body:`.

```ebnf
description := module_declaration                 -> {kind: "module_declaration", body: $1}
             | udp_declaration                    -> {kind: "udp_declaration", body: $1}
             | interface_declaration              -> {kind: "interface_declaration", body: $1}
             | program_declaration                -> {kind: "program_declaration", body: $1}
             | package_declaration                -> {kind: "package_declaration", body: $1}
             | attribute_instance* package_item   -> {kind: "package_item", attributes: $1, body: $2}
             | attribute_instance* bind_directive -> {kind: "bind_directive", attributes: $1, body: $2}
             | config_declaration                 -> {kind: "config_declaration", body: $1}
```

**Two layers of typed dispatch end-to-end** — `source_text_item.kind` (SV-Slice-3) routes to which top-level slot the item came from; `description.kind` (this slice) routes to which specific construct when the outer kind is `"description"`.

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — body field of the description-kind source_text_item was raw envelope:
"source_text": [
  {"kind": "description", "body": [<description Or-of-8 raw envelope>]}
]

# Post — body is itself a typed object with its own kind discriminator:
"source_text": [
  {
    "kind": "description",
    "body": {
      "kind": "module_declaration",
      "body": [<module_declaration envelope>]
    }
  }
]
```

**Consumer dispatch unlocked at the description level:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        match desc["kind"].as_str().unwrap() {
            "module_declaration"    => walk_module(&desc["body"]),
            "udp_declaration"       => walk_udp(&desc["body"]),
            "interface_declaration" => walk_interface(&desc["body"]),
            "program_declaration"   => walk_program(&desc["body"]),
            "package_declaration"   => walk_package(&desc["body"]),
            "package_item"          => walk_package_item(&desc["attributes"], &desc["body"]),
            "bind_directive"        => walk_bind_directive(&desc["attributes"], &desc["body"]),
            "config_declaration"    => walk_config(&desc["body"]),
            other => panic!("unknown description kind: {}", other),
        }
    }
}
```

**Annotation inventory:** 19 entries (was 11). 8 new per-branch entries on `description`.

**Inner `module_declaration`, `udp_declaration`, etc. shapes still raw envelope** — per-rule typing of those is a follow-up slice.

**Schema version:** stays at `1` (additive discriminator).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.4 / Contract 1.0.4 Highlights".

### 1.0.3 / Contract 1.0.3 — SV-Slice-3: `source_text_item` per-branch typed (`kind:` discriminator)

**What changed:** 8 per-branch annotations on `source_text_item` (lines 210-217 of `grammars/systemverilog.ebnf`). Every Or branch now emits a typed object with a `kind:` discriminator: `"description"`, `"local_parameter_declaration"`, `"parameter_declaration"`, `"package_import_declaration"`, `"timeunits_declaration"`, `"compiler_directive"`, `"comment_only_source_region"`, `"semi"`.

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — source_text[0] was the matched-branch shape directly:
"source_text": [
  [<description envelope>]
]

# Post — source_text[0] is a typed object with discriminator:
"source_text": [
  {"kind": "description", "body": [<description envelope>]}
]
```

**Consumer dispatch pattern:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "description" => walk_description(&item["body"]),
        "local_parameter_declaration" => walk_local_param(&item["body"]),
        "parameter_declaration" => walk_param(&item["body"]),
        "package_import_declaration" => walk_package_import(&item["body"]),
        "timeunits_declaration" => walk_timeunits(&item["body"]),
        "compiler_directive" => walk_compiler_directive(&item["body"]),
        "comment_only_source_region" => walk_comment_region(&item["body"]),
        "semi" => { /* stray ; — nothing to walk */ }
        other => panic!("unknown source_text_item kind: {}", other),
    }
}
```

**Annotation inventory:** 11 entries (was 3). 8 new per-branch entries on `source_text_item`.

**Trailing `semi` dropped** in the `local_parameter_declaration semi` and `parameter_declaration semi` branches — annotations reference `$1` only.

**`@branch_policy: priority_first` and `@priority` preserved** in the rule definition.

**Schema version:** stays at `1` (additive discriminator).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.3 / Contract 1.0.3 Highlights".

### 1.0.2 / Contract 1.0.2 — SV-Slice-2: `source_text` flatten-spread

**What changed:** `grammars/systemverilog.ebnf` line 2273's `source_text := source_text_item*` rule annotated `-> [$1**]`. The `source_text` field of `systemverilog_file` is now a flat array of `source_text_item` shapes (was a Quantified envelope).

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — source_text was nested Quantified envelope:
{
  "type": "systemverilog_file",
  "source_text": [<Quantified iteration wrap>]
}

# Post — source_text is a flat array (length 1 for minimal_module):
{
  "type": "systemverilog_file",
  "source_text": [<source_text_item shape>]
}
```

**Annotation inventory:** 3 entries (was 2). New: `source_text`.

**Annotation idiom:** `[$1**]` is the canonical flatten-spread form (same as regex.ebnf's `concatenation = piece+ -> [$1**]`). Verified to work for the SV grammar's first array-shaped rule.

**Schema version:** stays at `1` (additive — flat-array shape is a clean-up of the raw envelope).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.2 / Contract 1.0.2 Highlights".

### 1.0.1 / Contract 1.0.1 — SV-Slice-1: `systemverilog_file` typed (dangling annotation rescued)

**What changed:** `grammars/systemverilog.ebnf` line 184's `systemverilog_file` rule now carries its return annotation on the same multi-line definition (was dangling between the `sv_multi_entry_root` helper rule and `systemverilog_parseable_file`). The annotation `-> {type: "systemverilog_file", source_text: $2}` now correctly latches onto `systemverilog_file`. Same slice removed the `//` prefix from `systemverilog_parseable_file`'s annotation (PGEN's EBNF dialect uses `#` for comments, not `//`, so the `//` prefix was misleading rather than effective).

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre-SV-Slice-1 — recursive envelope:
{"content": {"Sequence": [
    {"content": {"Alternative": ...}, "rule_name": "element_0", ...},
    {"content": {"Alternative": ...}, "rule_name": "element_1", ...},
    ...
]}}

# Post-SV-Slice-1 — typed object at root:
{"content": {"Json": {
    "type": "systemverilog_file",
    "source_text": [...]
}}}
```

**Annotation inventory** (from `ast_pipeline`'s reporting): 2 entries (was 1). New: `systemverilog_file`. Existing: `systemverilog_parseable_file` (was already registered via the misleading `//` prefix; now registered via the documented path).

**Manifest update:** `rust/test_data/ast_shape_contract/systemverilog_v1.json` `current_content_kind` updated from placeholder `"sequence"` to calibrated `"json_object"`. Drift status flipped to `calibrated_2026_05_04`. Layout note about line 195 dangling annotation removed (resolved). Calibration history block added.

**Schema version:** stays at `1` (additive shape change within major version 1).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.1 / Contract 1.0.1 Highlights".

### 1.0.0 / Contract 1.0.0 — Foundation baseline (mdbook + contract Highlights structure)

**What changed:** Initial systemverilog mdBook scaffolded at `docs/systemverilog_parser_book/`. The integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` was upgraded from a thin "stable surface" pointer to the same release-tracked Highlights structure used by the regex parser contract.

**Mdbook chapters landed:** welcome, quickstart, build-recipe, public-api, ast-envelope, parse-content-variants, json-carrier, walking-the-ast, rules-top-level, examples-minimal-module, schema-versioning, glossary, changelog-index. Per-rule and per-feature example chapters land as the annotation campaign progresses.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.0 / Contract 1.0.0 Highlights".

**Build status:** Generated SV parser is **NOT in default `cargo test` build** — produced on-demand by `sv_stimuli_quality_gate`. See [Build Recipe](build-recipe.md).

**Annotation campaign:** Not yet started. `grammars/systemverilog.ebnf` is un-annotated except for one commented-out trial annotation at line 200. First slice will land in a follow-up commit.

**Schema baseline:** `1` (corresponds to `version: 1` in `rust/test_data/ast_shape_contract/systemverilog_v1.json`).

**Public API surface:** Unchanged. See [Public API Surface](public-api.md).

**Bug ledger:** No SV-NNNN entries blocking the baseline.

## How to track per-slice changes

Each annotation slice gets:

1. A grammar change in `grammars/systemverilog.ebnf` (the `-> ...` annotation).
2. A manifest update in `rust/test_data/ast_shape_contract/systemverilog_v1.json`.
3. A parser-release / contract-version bump in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the new schema version.
5. An entry in this changelog index summarising the slice.
6. A regression-lock test in `rust/src/embedding_api.rs` (or related test module) pinning the typed shape.

Per-slice commits should bundle all six (the live-book policy). See `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` for an example of a mature contract with 50+ Highlights sections to mirror.
