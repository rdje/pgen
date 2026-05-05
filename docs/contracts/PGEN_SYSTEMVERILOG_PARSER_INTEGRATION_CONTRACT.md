# docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's main `systemverilog` parser family.

This is the document downstream projects such as Nexsim should read first when deciding how to embed the PGEN systemverilog parser.

## Contract Identity
- Contract version:
  - `1.0.18`
- Parser release version:
  - `1.0.18`
- Embedding API contract baseline:
  - `1.2.0`
- SystemVerilog AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-04`
- Current grammar family label:
  - `systemverilog`
- Current stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Current live status:
  - Tracked in `LIVE_ACHIEVEMENT_STATUS.md`

## Current Trust Statement
- The PGEN `systemverilog` parser is **closure-grade for the current Nexsim-facing scope** when consumed through the stable `pgen::embedding_api` host surface.
- Closure is established via the family status / contract / telemetry gates listed under "Validation / Release Gates" below.
- The current sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- The grammar covers IEEE 1800-2017 (`sv_2017` profile) and the IEEE 1800-2023 delta (`sv_2023` profile). Both profiles share `grammars/systemverilog.ebnf` as the single source of truth.

## Companion Documentation — SystemVerilog Parser Integration mdBook
- The systemverilog-parser integration mdBook lives at `docs/systemverilog_parser_book/` and is the **canonical AST reference** for downstream consumers (Nexsim in particular).
- The book documents: build recipe, public API, the AST envelope, every annotated/un-annotated rule shape (as the annotation campaign progresses), per-feature worked examples, schema versioning, glossary, and a release-by-release index.
- Build it with `make systemverilog_parser_book_gate` (uses `mdbook build docs/systemverilog_parser_book`).
- Where the book and this contract disagree, **the contract wins** for compliance — but please report the disagreement as a documentation bug.

## Release 1.0.18 / Contract 1.0.18 Highlights — SV-Slice-18 batch: UDP truth-table entries typed

3 rules / 3 annotations completing the UDP truth-table walk path.

```ebnf
combinational_entry := level_input_list colon output_symbol semi
                    -> {inputs: $1, output: $3}

sequential_entry := seq_input_list colon current_state colon next_state semi
                 -> {inputs: $1, current_state: $3, next_state: $5}

udp_initial_statement := kw_initial output_port_identifier assign init_val semi
                      -> {name: $2, init_val: $4}
```

Every UDP truth-table row now exposes a clean typed shape — consumers walk `entries.first` and each `entries.rest` item directly without descending the raw envelope.

### Annotation inventory

122 entries (was 119). +3 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

## Release 1.0.17 / Contract 1.0.17 Highlights — SV-Slice-17 batch: UDP body sub-tree typed

6 rules / 8 annotations completing UDP declaration internals.

### Annotations

```ebnf
udp_body := combinational_body -> {kind: "combinational", body: $1}
         | sequential_body    -> {kind: "sequential",    body: $1}

udp_input_declaration := attribute_instance* kw_input list_of_udp_port_identifiers
                      -> {attributes: $1, identifiers: $3}

udp_output_declaration := attribute_instance* kw_output port_identifier
                            -> {kind: "wire", attributes: $1, name: $3}
                       | attribute_instance* kw_output kw_reg port_identifier (assign constant_expression)?
                            -> {kind: "reg", attributes: $1, name: $4, default: $5}

combinational_body := kw_table combinational_entry combinational_entry* kw_endtable
                   -> {entries: {first: $2, rest: $3}}

sequential_body := (udp_initial_statement)? kw_table sequential_entry sequential_entry* kw_endtable
                -> {initial: $1, entries: {first: $3, rest: $4}}

list_of_udp_port_identifiers := port_identifier (comma port_identifier)*
                             -> {first: $1, rest: $2}
```

### UDP declaration internals fully typed end-to-end

Combined with prior slices (UDP top-level rules from SV-Slice-12, port lists from SV-Slice-15), consumers walking a UDP `primitive ... endprimitive` construct get clean typed access at every level:

```rust
match desc.body.kind {
    "udp_declaration" => {
        let udp = &desc.body.body;
        match udp.kind {
            "ansi" | "nonansi" | "wildcard" | "extern_*" => {
                let header = &udp.header;          // {attributes, name, ports}
                let body = &udp.body;              // {kind, body} — combinational | sequential
                match body.kind {
                    "combinational" => {
                        let entries = &body.body.entries;  // {first, rest}
                        // walk combinational entries
                    }
                    "sequential" => {
                        let initial = &body.body.initial;  // optional udp_initial_statement
                        let entries = &body.body.entries;  // {first, rest}
                        // walk sequential entries
                    }
                }
            }
        }
    }
}
```

### Annotation inventory

119 entries (was 111). +8 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `combinational_entry`, `sequential_entry` (UDP truth-table entry sub-rules).
- `udp_initial_statement` typing.
- `package_or_generate_item_declaration` (large Or — 15+ branches).

## Release 1.0.16 / Contract 1.0.16 Highlights — SV-Slice-16 batch: port + port_direction + package_import family typed

4 rules / 9 annotations.

### Annotations

```ebnf
port := (port_expression)?
        -> {kind: "expression", expr: $1}
     | dot port_identifier lparen (port_expression)? rparen
        -> {kind: "named", name: $2, expr: $4}

port_direction := kw_input  -> {kind: "input"}
               | kw_output -> {kind: "output"}
               | kw_inout  -> {kind: "inout"}
               | kw_ref    -> {kind: "ref"}

package_import_declaration := kw_import package_import_item (comma package_import_item)* semi
                            -> {items: {first: $2, rest: $3}}

package_import_item := package_identifier scope_resolution identifier
                          -> {kind: "explicit", package: $1, name: $3}
                     | package_identifier scope_resolution star
                          -> {kind: "wildcard", package: $1}
```

### Notes

- **`port`** distinguishes positional ports `(expr)` from named-dot ports `.name(expr)`. Empty port placeholders (commas with no expression) flow through the `kind:"expression"` branch with `expr: []`.
- **`port_direction`** propagates as a typed sub-shape into any rule that references it (e.g., `ansi_port_declaration`'s named_dot branch's `direction:` field — when that rule eventually types).
- **`package_import_declaration`** wraps the `import a::*, b::c;` statement; consumers iterate `items.first + items.rest` for each import target.
- **`package_import_item`** discriminates `pkg::*` (wildcard) from `pkg::name` (explicit). Both `package` and `name` are clean identifier strings (inherited from SV-Slice-8).

### DEFERRED: `ansi_port_declaration` per-branch typing — task #38 blocker

Branch 0 (`( net_port_header | interface_port_header )? port_identifier ...`) starts with a parens-grouped Or. PGEN's annotation parser hits the parens-grouped-Or trailing-annotation attribution bug (task #38) — the per-branch annotations register out-of-order (branches 1+2 instead of 0+1+2) and the third branch's annotation is dropped entirely. Same blocker as `comment_only_source_region` from SV-Slice-6 batch. Reverted to un-annotated; tracked as follow-up either via task #38 fix OR grammar refactor extracting the leading parens-Or into a named helper rule.

### Annotation inventory

111 entries (was 102). +9 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `package_or_generate_item_declaration` Or (large — 15+ branches; reaches deep into the SV grammar's declaration tree).
- `port_expression` per-branch typing.
- `udp_output_declaration` / `udp_input_declaration` per-branch typing.
- Grammar refactor or task #38 fix to unblock `ansi_port_declaration`, `comment_only_source_region`, and other parens-grouped-Or rules.

## Release 1.0.15 / Contract 1.0.15 Highlights — SV-Slice-15 batch: port-list family + small structural rules typed

6 rules / 7 annotations. Every `header.ports` field on every typed module/interface/program/UDP declaration now surfaces a typed shape instead of the raw envelope.

### Annotations

```ebnf
list_of_ports := lparen port (comma port)* rparen
              -> {first: $2, rest: $3}

list_of_port_declarations := lparen (attribute_instance* ansi_port_declaration (comma attribute_instance* ansi_port_declaration)*)? rparen
                          -> $2

udp_port_list := output_port_identifier comma input_port_identifier (comma input_port_identifier)*
              -> {output: $1, inputs: {first: $3, rest: $4}}

udp_declaration_port_list := udp_output_declaration comma udp_input_declaration (comma udp_input_declaration)*
                          -> {output: $1, inputs: {first: $3, rest: $4}}

anonymous_program := kw_program semi anonymous_program_item* kw_endprogram
                  -> {items: $3}

package_export_declaration := kw_export star scope_resolution star semi
                                 -> {kind: "wildcard"}
                            | kw_export package_import_item (comma package_import_item)* semi
                                 -> {kind: "explicit", items: {first: $2, rest: $3}}
```

### Notes per rule

- **`list_of_ports` and `list_of_port_declarations` differ in shape**: the former emits `{first, rest}` (mini-mixed-array workaround for `port (comma port)*`); the latter passes the optional inner content through transparently with `-> $2` (the parens-grouped optional sequence). `list_of_port_declarations` body when populated is a 3-element envelope `[<attribute_instance*>, <ansi_port_declaration>, <(comma attribute_instance* ansi_port_declaration)*>]`. Per-rule typing of `ansi_port_declaration` is a follow-up slice.
- **`udp_port_list` vs `udp_declaration_port_list`** parallel shapes (output + inputs.{first, rest}) but the underlying sub-rules differ — `udp_port_list` uses identifier strings (`output_port_identifier`, `input_port_identifier`), `udp_declaration_port_list` uses full declarations (`udp_output_declaration`, `udp_input_declaration`).
- **`anonymous_program`** drops kw_program/semi/kw_endprogram and exposes only `items`. Reachable via `package_item.kind = "anonymous_program"` then walk `body.items`.
- **`package_export_declaration`** discriminates between wildcard `export *::*;` and explicit `export item, item, ...;`. Wildcard form drops everything (just the kind label). Explicit form uses the standard {first, rest} mini-mixed-array.

### Annotation inventory

102 entries (was 95). +7 in this batch. **Crossing 100 annotations** for the SV grammar — the campaign is now mid-flight.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `port` per-branch typing (the inner element of `list_of_ports`).
- `ansi_port_declaration` per-branch typing (the inner element of `list_of_port_declarations`).
- `udp_output_declaration` / `udp_input_declaration` per-branch typing.
- `package_or_generate_item_declaration` (large Or — the actual content under package_item.kind = "declaration"; reaches deep into the SV grammar).
- `package_import_declaration` / `package_import_item` typing.

## Release 1.0.14 / Contract 1.0.14 Highlights — SV-Slice-14 batch: bind sub-tree completion + interface_class_declaration + config_declaration

5 rules typed in one batch — completes the bind directive sub-tree (started in SV-Slice-13) and adds two more top-level construct families.

### Annotations

```ebnf
bind_target_scope := module_identifier    -> {kind: "module",    name: $1}
                  | interface_identifier -> {kind: "interface", name: $1}

bind_target_instance := hierarchical_identifier constant_bit_select
                     -> {name: $1, bit_select: $2}

bind_target_instance_list := bind_target_instance (comma bind_target_instance)*
                          -> {first: $1, rest: $2}

interface_class_declaration := kw_interface kw_class declared_interface_class_identifier
                                (parameter_port_list)?
                                (kw_extends interface_class_type (comma interface_class_type)*)?
                                semi interface_class_item* kw_endclass (colon class_identifier)?
                            -> {name: $3, parameters: $4, extends: $5, items: $7, end_label: $9}

config_declaration := kw_config config_identifier semi
                       (local_parameter_declaration semi)*
                       design_statement
                       config_rule_statement*
                       kw_endconfig (colon config_identifier)?
                   -> {name: $2, local_params: $4, design: $5, rules: $6, end_label: $8}
```

### Bind sub-tree fully typed

Combined with SV-Slice-13's bind_directive/bind_instantiation typing, consumers walking a bind directive get clean typed access at every level:

```rust
// description.kind = "bind_directive" → desc.body is the typed bind shape
match desc.body.kind {
    "scoped" => {
        // bind <target_scope> [: <instances>] <instantiation>
        let scope = &desc.body.target_scope;     // {kind, name} from bind_target_scope
        let instances = &desc.body.instances;    // {first, rest} from bind_target_instance_list (or [] if no `:` clause)
        let inst = &desc.body.instantiation;     // {kind, body} from bind_instantiation
        match scope.kind { "module" | "interface" => /* ... */ }
        // ... iterate instances.first + instances.rest with each as
        //     {name, bit_select} from bind_target_instance
    }
    "single" => {
        // bind <target_instance> <instantiation>
        let inst = &desc.body.target_instance;   // {name, bit_select}
        // ...
    }
}
```

### `interface_class_declaration` and `config_declaration`

Both are single-sequence rules (no Or branches) typed with named fields. Reachable via `package_item.kind = "declaration"` (then walk into the package_or_generate_item_declaration body) for interface_class_declaration; via `description.kind = "config_declaration"` for config_declaration.

### Annotation inventory

95 entries (was 89). +6 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

**`{first, rest}` workaround applied a third time** (after attribute_instance and udp_port_decls) — for `bind_target_instance_list`'s `X (comma X)*` mini-mixed-array. The pattern is now firmly established for any "required-first + repeat" rule shape. A future codegen extension supporting true `[$1, $2**]` mixed-array spread would let these all flatten to clean arrays.

### Next slice candidates

- `bind_target_instance.bit_select` deep typing (constant_bit_select sub-rule).
- `udp_port_list` / `udp_declaration_port_list` (sub-rule typing inside `header.ports` for UDP).
- `list_of_ports` / `list_of_port_declarations` (sub-rule typing for module/interface/program port lists).
- `package_or_generate_item_declaration` (large Or — the actual content under package_item.kind = "declaration").
- `package_export_declaration`, `anonymous_program` per-branch typing.

## Release 1.0.13 / Contract 1.0.13 Highlights — SV-Slice-13 batch: bind_directive + bind_instantiation + package_item per-branch typed

3 Or rules typed; downstream consumers gain clean kind dispatch on description's `package_item` and `bind_directive` branches (reached when description.kind = `"package_item"` or `"bind_directive"`).

### Annotations

```ebnf
bind_directive := kw_bind bind_target_scope (colon bind_target_instance_list)? bind_instantiation semi
                    -> {kind: "scoped", target_scope: $2, instances: $3, instantiation: $4}
               | kw_bind bind_target_instance bind_instantiation semi
                    -> {kind: "single", target_instance: $2, instantiation: $3}

bind_instantiation := program_instantiation   -> {kind: "program",   body: $1}
                   | module_instantiation     -> {kind: "module",    body: $1}
                   | interface_instantiation  -> {kind: "interface", body: $1}
                   | checker_instantiation    -> {kind: "checker",   body: $1}

package_item := package_or_generate_item_declaration -> {kind: "declaration",        body: $1}
             | anonymous_program                     -> {kind: "anonymous_program",  body: $1}
             | package_export_declaration            -> {kind: "export",             body: $1}
             | timeunits_declaration                 -> {kind: "timeunits",          body: $1}
```

### Consumer dispatch

```rust
// description.kind == "bind_directive" → desc.body is the typed bind_directive shape
match desc.body.kind {
    "scoped" => {
        // (?<scope> : <instances>)? <instantiation>
        let scope = &desc.body.target_scope;
        let instances = &desc.body.instances;  // empty array if no `:` clause
        let inst = &desc.body.instantiation;
        process_bind_scoped(scope, instances, inst);
    }
    "single" => {
        // <target_instance> <instantiation>
        process_bind_single(&desc.body.target_instance, &desc.body.instantiation);
    }
}

// inst.kind dispatches to which form of instantiation:
match inst.kind {
    "program" | "module" | "interface" | "checker" => walk_instantiation(inst.kind, &inst.body),
}

// description.kind == "package_item" → desc.body is the typed package_item shape
match desc.body.kind {
    "declaration"       => process_decl(&desc.body.body),
    "anonymous_program" => process_anon_program(&desc.body.body),
    "export"            => process_export(&desc.body.body),
    "timeunits"         => process_timeunits(&desc.body.body),
}
```

### Annotation inventory

89 entries (was 79). +10 in this batch.

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Next slice candidates

- `bind_target_scope` (2-form Or — module_identifier vs interface_identifier).
- `bind_target_instance` and `bind_target_instance_list` (single-sequence + comma-spread mini-mixed-array).
- `interface_class_declaration` per-branch.
- `config_declaration` (single sequence, ~10 elements).
- Sub-rule typing inside `header.ports` (`udp_port_list`, `udp_declaration_port_list`, `list_of_ports`).

## Release 1.0.12 / Contract 1.0.12 Highlights — SV-Slice-12 batch: UDP declaration family typed (mirror of module/interface/program pattern + mini-mixed-array workaround)

> **For Nexsim maintainers:** UDP (User-Defined Primitive) declarations now have the same typed surface as module/interface/program. 4-layer typed dispatch end-to-end for `primitive p (...) ... endprimitive` constructs reachable from `description.body` when `kind:"udp_declaration"`.

### Annotations

```ebnf
udp_ansi_declaration := attribute_instance* kw_primitive udp_identifier lparen udp_declaration_port_list rparen semi
                     -> {attributes: $1, name: $3, ports: $5}

udp_nonansi_declaration := attribute_instance* kw_primitive udp_identifier lparen udp_port_list rparen semi
                        -> {attributes: $1, name: $3, ports: $5}

udp_declaration_sv_2017 := udp_nonansi_declaration udp_port_declaration udp_port_declaration* udp_body kw_endprimitive (colon udp_identifier)?
                            -> {kind: "nonansi", header: $1, port_decls: {first: $2, rest: $3}, body: $4, end_label: $6}
                         | udp_ansi_declaration udp_body kw_endprimitive (colon udp_identifier)?
                            -> {kind: "ansi", header: $1, body: $2, end_label: $4}
                         | kw_extern udp_nonansi_declaration
                            -> {kind: "extern_nonansi", header: $2}
                         | kw_extern udp_ansi_declaration
                            -> {kind: "extern_ansi", header: $2}
                         | attribute_instance* kw_primitive udp_identifier lparen dot_star rparen semi udp_port_declaration* udp_body kw_endprimitive (colon udp_identifier)?
                            -> {kind: "wildcard", attributes: $1, name: $3, port_decls: $8, body: $9, end_label: $11}

udp_declaration_sv_2023 := /* same 5 branches; wildcard branch positional shift for `dot star` (2 tokens) vs `dot_star` (1 token) → port_decls $8→$9, body $9→$10, end_label $11→$12 */
```

### `port_decls: {first, rest}` mini-mixed-array workaround

The `nonansi` branch (branch 0) has the pattern `udp_port_declaration udp_port_declaration*` — a required first port-decl followed by zero-or-more reps. Mixed-array spread `[$2, $3**]` is currently blocked by the annotation-language limitation (per `feedback_annotation_no_mixed_spread.md`), so the typed shape uses the `{first, rest}` workaround (same idiom as `attribute_instance` from SV-Slice-6). Consumers walking `port_decls` for `kind:"nonansi"` should:

```rust
let port_decls = &udp["port_decls"];
process_port_decl(&port_decls["first"]);
for rest_item in port_decls["rest"].as_array().unwrap() {
    // rest_item is a [matched_iteration] envelope of udp_port_declaration
    process_port_decl(rest_item);
}
```

For `kind:"wildcard"`, `port_decls` is a plain `[]`-iteration array (no leading port; handled identically to module/interface wildcard).

### 5 kind labels

- `nonansi` — `udp_nonansi_declaration` form with port-decl block
- `ansi` — `udp_ansi_declaration` form
- `wildcard` — `(.*)` form (UDP variant)
- `extern_nonansi`, `extern_ansi` — extern declarations

### Annotation inventory

79 entries (was 67). +12 in this batch: 1 (udp_ansi_declaration) + 1 (udp_nonansi_declaration) + 5 (udp_declaration_sv_2017) + 5 (udp_declaration_sv_2023).

### Same accept set, same diagnostic codes. Schema stays at `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

**`{first, rest}` workaround for `X X*` mini-mixed-array** — used here for `port_decls: {first: $2, rest: $3}`. Same idiom as `attribute_instance: {first, rest}` from SV-Slice-6. Until the annotation language gains true mixed-array spread (`[$2, $3**]`), this is the canonical pattern for "required-first + repeat" rule shapes.

### Next slice candidates

- `interface_class_declaration` per-branch (sibling to class_declaration).
- `program_ansi_header` / `program_nonansi_header` (already done in SV-Slice-11).
- `udp_port_list` / `udp_declaration_port_list` (sub-rule typing inside `header.ports`).
- `udp_body` / `udp_port_declaration` (sub-rules inside the typed UDP shape).
- `description` further branches: `package_item`, `bind_directive`, `config_declaration`.

## Release 1.0.11 / Contract 1.0.11 Highlights — SV-Slice-11 batch: program-header sub-tree typed (mirror of module/interface header pattern)

2 rules typed: `program_ansi_header`, `program_nonansi_header`. Both use the same field-name set as `module_ansi_header` / `interface_ansi_header` (sans `keyword:` since program only has one keyword).

### Annotations

```ebnf
program_ansi_header := attribute_instance* kw_program_81d9aeea (lifetime)? program_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                    -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

program_nonansi_header := attribute_instance* kw_program_81d9aeea (lifetime)? program_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                       -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

### Empirical verification on `program p; endprogram\n`

```text
description.body.body (program_declaration_sv_2017 ANSI form):
  kind: "ansi"
  header:
    attributes: []
    lifetime: []
    name: "p"          # clean string (inherited from SV-Slice-8)
    imports: []
    parameters: []
    ports: []
  timeunits: []
  items: []
  end_label: []
```

### Sibling-rule symmetry

The 3 top-level construct families that have ANSI/non-ANSI header pairs (module / interface / program) all expose the same 6-7 field shape (`attributes`, `keyword?`, `lifetime`, `name`, `imports`, `parameters`, `ports`). Consumers can write a single header walker that handles all three families.

### Annotation inventory

67 entries (was 65). +2 in this batch.

### Same accept set, same diagnostic codes.

### Schema-version stays `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

**Sibling-rule pattern reuse**: when a family of rules shares structure (here, ansi/nonansi header pairs across module/interface/program), reusing the same field-name set across them is intentional and lets consumers write generic walkers. Module headers have an extra `keyword:` field for module/macromodule disambiguation; interface and program don't (single keyword each).

### Next slice candidates

- `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.
- `udp_ansi_declaration` / `udp_nonansi_declaration` per-branch (UDP has its own ANSI/non-ANSI distinction).
- Investigation: package top-level parse failure.

## Release 1.0.10 / Contract 1.0.10 Highlights — SV-Slice-10 batch: class + package + program declarations typed

5 rules typed: class declarations (sv_2017 + sv_2023 single-sequence shapes), `package_declaration` single-sequence, program declarations (sv_2017 + sv_2023 — 5 per-branch kinds each, mirroring module/interface pattern).

### Annotations

```ebnf
class_declaration_sv_2017 := (kw_virtual)? kw_class (lifetime)? declared_class_identifier (parameter_port_list)? (kw_extends base_class_type (lparen list_of_arguments rparen)?)? (kw_implements interface_class_type (comma interface_class_type)*)? semi class_item* kw_endclass (colon class_identifier)?
                          -> {virtual: $1, lifetime: $3, name: $4, parameters: $5, extends: $6, implements: $7, items: $9, end_label: $11}

class_declaration_sv_2023 := (kw_virtual)? kw_class (final_specifier)? declared_class_identifier (parameter_port_list)? (kw_extends base_class_type (lparen (list_of_arguments | kw_default)? rparen)?)? (kw_implements interface_class_type (comma interface_class_type)*)? semi class_item* kw_endclass (colon class_identifier)?
                          -> {virtual: $1, final_specifier: $3, name: $4, parameters: $5, extends: $6, implements: $7, items: $9, end_label: $11}

package_declaration := attribute_instance* kw_package (lifetime)? package_identifier semi (timeunits_declaration)? (attribute_instance* package_item)* kw_endpackage (colon package_identifier)?
                    -> {attributes: $1, lifetime: $3, name: $4, timeunits: $6, items: $7, end_label: $9}

program_declaration_sv_2017 := /* 5 branches, kind: nonansi/ansi/wildcard/extern_nonansi/extern_ansi
                                  Note: nonansi listed BEFORE ansi (different from module/interface order),
                                        but kind labels still discriminate correctly. */

program_declaration_sv_2023 := /* same 5 branches; wildcard branch positional shift for `dot star` vs `dot_star`. */
```

### Profile-specific field naming on class declarations

The class rule's `lifetime` slot in SV-2017 became `final_specifier` in SV-2023 (different LRM semantics). The annotation reflects this — sv_2017 carries `lifetime: $3`, sv_2023 carries `final_specifier: $3`. Consumers walking either profile dispatch on the present field name; both fields are mutually exclusive across profiles.

### Empirical verification

| Input | Outcome |
|---|---|
| `module m; endmodule\n` | ✓ unchanged (module pattern preserved) |
| `interface bus; endinterface\n` | ✓ unchanged (interface pattern preserved) |
| `program p; endprogram\n` | ✓ NEW — `description.body.kind = "program_declaration"`, `description.body.body.kind = "ansi"` |
| `package p; endpackage\n` | ✗ parse rejected at position 0 — annotation registered correctly per the inventory; runtime parse failure appears pre-existing (this slice's annotation didn't introduce it; module/interface/program tests still pass with the same regenerated parser). Investigation tracked separately. |
| `class C; endclass\n` | ✗ expected — class_declaration is not directly in source_text_item's reachable set; class declarations are typically reached through `package_item` or other subsidiary rules. |

### Annotation inventory

65 entries (was 53). +12 in this batch: 1 (class_declaration_sv_2017) + 1 (class_declaration_sv_2023) + 1 (package_declaration) + 5 (program_declaration_sv_2017) + 5 (program_declaration_sv_2023) — but note that package_declaration's runtime path needs investigation despite the annotation registering correctly.

### Same accept set, same diagnostic codes

(Verified: module/interface/program inputs that worked before still work; the 65-annotation parser is correct for those.)

### Schema-version stays `1`.

### mdBook updates, gate green.

### Annotation-language idiom note

**Single-sequence rule typing** (no kind discriminator) is appropriate for rules that have only one form, like `class_declaration_sv_2017` and `package_declaration`. They emit a flat object with named fields rather than a `kind`-discriminated shape. Consumers reach them via the parent's `description.kind` (e.g. "class_declaration", "package_item" → contains class/package; "program_declaration" → 5-form discriminator).

### Open follow-up

- Investigate why `package mypkg; endpackage\n` doesn't parse at top level despite `package_declaration` being in `description`'s Or set. Module / interface / program with similar structures parse fine. Could be (i) a pre-existing PEG ordering issue, (ii) interaction with the `@emit_fact:` rule-level metadata annotation immediately preceding `package_declaration`, or (iii) a different rule-context constraint not visible from inspection. Tracked in MEMORY.md as a separate item.

### Next slice candidates

- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch (deferred from this batch — has `udp_port_declaration udp_port_declaration*` mini-mixed-array pattern that needs the `{first, rest}` workaround).
- Type `program_ansi_header` / `program_nonansi_header` (sibling to `module_<form>_header`).
- Type `interface_keyword`, `kw_interface`, `kw_class`, `kw_package` (clean keyword strings — minor polish).
- Address task #38 to unblock parens-grouped-Or rules.

## Release 1.0.9 / Contract 1.0.9 Highlights — SV-Slice-9 batch: interface declarations typed (full mirror of module pattern)

Interface declarations now have the same typed surface as module declarations. 4-layer typed dispatch end-to-end: `source_text_item.kind` → `description.kind` → `interface_declaration_sv_<profile>.kind` → `interface_<form>_header.name` (clean string).

### Annotations

```ebnf
interface_ansi_header := attribute_instance* kw_interface_5ea2d81a (lifetime)? interface_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                      -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

interface_nonansi_header := attribute_instance* kw_interface_5ea2d81a (lifetime)? interface_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                         -> {attributes: $1, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

interface_declaration_sv_2017 := interface_ansi_header (timeunits_declaration)? non_port_interface_item* kw_endinterface_ebd6ca35 (colon interface_identifier)?
                                  -> {kind: "ansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                              | interface_nonansi_header (timeunits_declaration)? interface_item* kw_endinterface_ebd6ca35 (colon interface_identifier)?
                                  -> {kind: "nonansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                              | attribute_instance* kw_interface_5ea2d81a interface_identifier lparen dot_star rparen semi (timeunits_declaration)? interface_item* kw_endinterface_ebd6ca35 (colon interface_identifier)?
                                  -> {kind: "wildcard", attributes: $1, name: $3, timeunits: $8, items: $9, end_label: $11}
                              | kw_extern_bf1ee311 interface_nonansi_header
                                  -> {kind: "extern_nonansi", header: $2}
                              | kw_extern_bf1ee311 interface_ansi_header
                                  -> {kind: "extern_ansi", header: $2}

interface_declaration_sv_2023 := /* same 5 branches; wildcard branch's positional indices shift to $9/$10/$12 because dot star (2 tokens) vs dot_star (1 token) */
```

### Differences from module pattern

- **No `keyword:` field on interface_<form>_header** — interface only has one keyword (`interface`), unlike module which has both `module` and `macromodule`. The kind discriminator at the parent level (description.kind == "interface_declaration") fully identifies the construct; an inner keyword field would be redundant. (Module headers expose `keyword: {kind: "module"|"macromodule"}` for that distinction.)
- **Same field names otherwise** — `attributes`, `lifetime`, `name`, `imports`, `parameters`, `ports` for headers; `kind`, `header`, `timeunits`, `items`, `end_label` (and `attributes`, `name` on wildcard) for declaration-level. Consumer dispatch code can mostly share between modules and interfaces.

### Empirical verification on `interface bus; endinterface\n`

```text
source_text[0]:
  kind: "description"
  body:
    kind: "interface_declaration"
    body:
      kind: "ansi"
      header:
        name: "bus"            # clean string (inherited from SV-Slice-8)
        attributes: []
        lifetime: []
        imports: []
        parameters: []
        ports: []
      timeunits: []
      items: []
      end_label: []
```

### Annotation inventory

53 entries (was 41). +12 in this batch.

### Same accept set, same diagnostic codes.

### Schema-version stays `1`.

### mdBook updated, gate green.

### Annotation-language idiom note

This slice demonstrates **structural reuse of the module typing pattern** for a sibling rule family. Same kind labels (ansi/nonansi/wildcard/extern_nonansi/extern_ansi), same field names where they apply. Consumer code sharing between module and interface walkers: trivial.

### Next slice candidates

- Type `class_declaration_sv_2017` / `class_declaration_sv_2023` per-branch.
- Type `package_declaration` (single sequence, attribute_instance* prefix).
- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.
- Type `program_declaration_sv_2017` / `program_declaration_sv_2023` per-branch.
- Type `kw_interface_5ea2d81a` / `kw_class_8d767bf5` / etc. (clean keyword strings) — minor but adds polish if needed for downstream tools.

## Release 1.0.8 / Contract 1.0.8 Highlights — SV-Slice-8 batch: identifier-leaf rules typed (clean strings propagate through every identifier-typed field)

This is the highest-leverage slice yet — typing 4 leaf rules causes clean identifier strings to propagate through every rule that resolves to an identifier (every `*_identifier` alias, every name field of every typed parent rule).

### Annotations

```ebnf
simple_identifier := trivia /[a-zA-Z_][a-zA-Z0-9_$]*/
                  -> $2

@sample: "\\foo "
escaped_identifier := trivia /\\[!-~]+/
                   -> $2

non_keyword_identifier := !reserved_non_keyword_identifier identifier
                       -> $2

@sample: "foo"
simple_identifier_no_scope := trivia /[a-zA-Z_][a-zA-Z0-9_$]*(?![ \t\r\n]*::)/
                           -> $2
```

All four use the `-> $2` transparent-passthrough idiom (drop trivia / lookahead, surface the regex-captured identifier name as a clean string).

### Propagation chain

```text
simple_identifier (typed: -> $2)
  ↓ matches → "m"
identifier := escaped_identifier | simple_identifier
  ↓ transparent Or → "m"
non_keyword_identifier := !reserved_non_keyword_identifier identifier (typed: -> $2)
  ↓ drops lookahead, surfaces identifier → "m"
declaration_identifier := non_keyword_identifier
  ↓ transparent alias → "m"
module_identifier := declaration_identifier
  ↓ transparent alias → "m"
class_identifier, package_identifier, etc.
  ↓ transparent aliases → "m"
```

Every typed parent rule that exposes an identifier-typed field now surfaces it as a clean JSON string. For `module_ansi_header.name`, `module_nonansi_header.name`, `description.body.body.wildcard.name` (the `(.*)` form's name field), and any future typed rule referencing `*_identifier`, the field is a clean string.

### Empirical pre/post on `module m; endmodule\n`

```text
# Pre-SV-Slice-8 — header.name was raw envelope:
"header": {"keyword": {"kind": "module"}, "name": [[], "m"], "lifetime": [], ...}

# After SV-Slice-8a (just simple_identifier + escaped_identifier typed):
"header": {"keyword": {"kind": "module"}, "name": [[], "m"], "lifetime": [], ...}
                                                  ↑ still wrapped — non_keyword_identifier still raw

# Post-SV-Slice-8 (full batch — all 4 leaf rules typed):
"header": {"keyword": {"kind": "module"}, "name": "m", "lifetime": [], ...}
                                                  ↑ clean string!
```

### Why this slice is the highest-leverage so far

Typing 4 leaf rules causes EVERY identifier in EVERY future-typed rule to land as a clean string with zero additional annotation work. Future slices typing `interface_declaration.name`, `class_declaration.name`, `package_declaration.name`, `signal_identifier`, `port_identifier`, `parameter_identifier`, etc. — all get clean strings automatically. This is dependency-graph-leveraged annotation work: type the dependency once, every dependent benefits.

### Annotation inventory

41 entries (was 37). +4 in this batch.

### Notes on the lookahead positional slot

PGEN's annotation language treats negative lookaheads (`!X`) as occupying positional slots even though they don't consume tokens. So in `non_keyword_identifier := !reserved_non_keyword_identifier identifier`, `$1` is the (empty) lookahead slot and `$2` is the matched `identifier`. Same convention as the regex parser used for `simple_escape = !"o{" !"x{" !"p{" !"P{" any_char -> {... char: $5}` (4 lookaheads → `$5` for the consumer).

### Same accept set, same diagnostic codes.

### Schema-version stays `1` (additive).

### mdBook

`changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.

### Public API surface unchanged.

### Annotation-language idiom note

**Leaf-rule trivia-stripping with `-> $N`** (where `$N` is the regex-capture position past the trivia prefix) is the canonical pattern for surfacing clean text from `trivia /regex/` rules. Applied here to 4 identifier-leaf rules; same idiom used in regex parser (`hex_digits`, `prop_name`, etc.) and earlier in this campaign on `compiler_directive`.

### Next slice candidates

- Type `class_declaration_sv_2017` per-branch (mirror of module_declaration's pattern; class declarations now have clean name strings).
- Type `interface_declaration_sv_2017` / `interface_declaration_sv_2023` per-branch.
- Type `package_declaration` (substantial — single sequence with attribute_instance* prefix; identifier name is now clean, simplifying the typed shape).
- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.
- Type `program_declaration_sv_2017` / `program_declaration_sv_2023`.

## Release 1.0.7 / Contract 1.0.7 Highlights — SV-Slice-7 batch: module_keyword + lifetime + module_ansi_header + module_nonansi_header typed (4 layers of dispatch end-to-end)

Typing the `header:` field that the SV-Slice-6 batch left as raw envelope. Four sub-rules typed in one pass; **four layers of typed dispatch are now end-to-end** for module declarations.

### (a) `module_keyword` per-branch (2 kind labels)

```ebnf
module_keyword := kw_module_fbd34a2b      -> {kind: "module"}
                | kw_macromodule_df04b866 -> {kind: "macromodule"}
```

Drops the keyword token (it's redundant with `kind`); emits a typed object that consumers can dispatch on.

### (b) `lifetime` per-branch (2 kind labels)

```ebnf
lifetime := kw_static_a381562a    -> {kind: "static"}
          | kw_automatic_ebe88724 -> {kind: "automatic"}
```

Same pattern as module_keyword. When a `(lifetime)?` slot is matched, consumers see `{kind: "static"}` / `{kind: "automatic"}`. When un-matched, they see `[]` (existing convention).

### (c) `module_ansi_header` typed sequence

```ebnf
module_ansi_header := attribute_instance* module_keyword ( lifetime )? module_identifier package_import_declaration* ( parameter_port_list )? ( list_of_port_declarations )? semi
                   -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

7 named fields. Drops the trailing `semi`. The `keyword:` field is itself typed (per slice 7a), and `lifetime:` is itself typed when matched (per slice 7b). `name:` carries the raw `module_identifier` envelope (still un-typed; per-rule typing of identifiers is follow-up). `attributes`/`imports`/`parameters`/`ports` are quantified or optional; consumers handle empty as `[]` and matched as the inner shape.

### (d) `module_nonansi_header` typed sequence

```ebnf
module_nonansi_header := attribute_instance* module_keyword ( lifetime )? module_identifier package_import_declaration* ( parameter_port_list )? list_of_ports semi
                      -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

Same field names as `module_ansi_header`. Consumers walking either get the same JSON shape — only the `ports:` source rule differs (`list_of_ports` vs `(list_of_port_declarations)?`). For consumer code, walking the typed shape is uniform across ANSI / non-ANSI forms.

### Empirical pre/post on `module m; endmodule\n` (sv_2017 profile)

```text
# Pre-SV-Slice-7 — header was raw envelope:
"body": {
  "kind": "ansi",
  "header": [<module_ansi_header raw 8-element envelope>],
  "timeunits": [],
  "items": [],
  "end_label": []
}

# Post-SV-Slice-7 — header is itself a typed object with named fields:
"body": {
  "kind": "ansi",
  "header": {
    "attributes": [],
    "keyword": {"kind": "module"},
    "lifetime": [],
    "name": [<module_identifier raw envelope>],
    "imports": [],
    "parameters": [],
    "ports": []
  },
  "timeunits": [],
  "items": [],
  "end_label": []
}
```

### Four layers of typed dispatch end-to-end

```rust
// Walking a module declaration end-to-end:
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        if desc["kind"] == "module_declaration" {
            let module = &desc["body"];   // module_declaration_sv_<profile> shape
            match module["kind"].as_str().unwrap() {
                "ansi" | "nonansi" => {
                    let hdr = &module["header"];   // module_<form>_header shape
                    let module_kind = hdr["keyword"]["kind"].as_str().unwrap();   // "module" | "macromodule"
                    let lifetime = match &hdr["lifetime"] {
                        v if v.is_array() && v.as_array().unwrap().is_empty() => None,
                        v => Some(v["kind"].as_str().unwrap()),  // "static" | "automatic"
                    };
                    let attrs = hdr["attributes"].as_array().unwrap();
                    let imports = hdr["imports"].as_array().unwrap();
                    // ... process module ...
                }
                "wildcard" => { /* similar — wildcard exposes more fields directly */ }
                "extern_nonansi" | "extern_ansi" => {
                    let hdr = &module["header"];   // same module_<form>_header shape
                    // ... process extern declaration ...
                }
                _ => unreachable!(),
            }
        }
    }
}
```

### Annotation inventory

37 entries (was 31). +6 in this batch: 2 (module_keyword) + 2 (lifetime) + 1 (module_ansi_header) + 1 (module_nonansi_header).

### Same accept set, same diagnostic codes.

### Schema-version stays `1` (additive).

### mdBook

`changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.

### Public API surface unchanged.

### Annotation-language idiom notes

- **Tiny-Or-typed-as-kind-tag** (`X := A -> {kind: "a"} | B -> {kind: "b"}`) is the regex-campaign pattern for keyword-distinguishing rules. Used here on `module_keyword` and `lifetime`. Once a keyword rule is typed this way, every parent rule that references it inherits the typed sub-shape automatically.

### Next slice candidates

- Type `module_identifier` / `declaration_identifier` (currently the un-typed `name:` field on module_<form>_header).
- Type `class_declaration_sv_2017` / `class_declaration_sv_2023` per-branch (mirror of module_declaration pattern).
- Type `interface_declaration_sv_2017` / `interface_declaration_sv_2023` per-branch.
- Type `package_declaration` (substantial sequence with attribute_instance* prefix).
- Type `udp_declaration_sv_2017` / `udp_declaration_sv_2023` per-branch.

## Release 1.0.6 / Contract 1.0.6 Highlights — SV-Slice-6 batch: attribute_instance + module_declaration_sv_2017/2023 typed (3 layers of dispatch end-to-end)

This is a multi-rule batch slice typing 3 rules in one pass. Three layers of typed dispatch are now end-to-end: `source_text_item.kind` → `description.kind` → `module_declaration_sv_<profile>.kind`.

### (a) `attribute_instance` — `{first, rest}` shape

```ebnf
attribute_instance := attr_open attr_spec ( comma attr_spec )* attr_close
                   -> {first: $2, rest: $3}
```

Drops the `attr_open` (`(*`) and `attr_close` (`*)`) syntactic delimiters. Exposes the first attr_spec as `first:` and the trailing `( comma attr_spec )*` repetitions as `rest:` (each rest entry is `[comma, attr_spec]`). Mixed-array spread `[$2, $3**]` is currently blocked by an annotation-language limitation (per `feedback_annotation_no_mixed_spread.md`) so the cleaner flat-array form is deferred. Consumers walk `obj.first` for the leading attribute and iterate `obj.rest` for additional attributes.

### (b) `module_declaration_sv_2017` per-branch typed (5 forms)

```ebnf
module_declaration_sv_2017 := @sample: "module m; endmodule" module_ansi_header (timeunits_declaration)? non_port_module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "ansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(a); endmodule" module_nonansi_header (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "nonansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(.*); endmodule" attribute_instance* module_keyword (lifetime)? module_identifier lparen dot_star rparen semi (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "wildcard", attributes: $1, keyword: $2, lifetime: $3, name: $4, timeunits: $9, items: $10, end_label: $12}
                            | @sample: "extern module m(a);" kw_extern_bf1ee311 module_nonansi_header
                                -> {kind: "extern_nonansi", header: $2}
                            | @sample: "extern module m;" kw_extern_bf1ee311 module_ansi_header
                                -> {kind: "extern_ansi", header: $2}
```

5 kind labels: `"ansi"`, `"nonansi"`, `"wildcard"`, `"extern_nonansi"`, `"extern_ansi"`. Each carries the structured fields needed to walk the matched form. The wildcard branch (the `(.*)` form) preserves the leading `attribute_instance*`, the `module_keyword`, optional `lifetime`, and the `module_identifier` as named fields. The two extern branches expose only the matched header as a `header:` field (drops the `kw_extern` keyword).

### (c) `module_declaration_sv_2023` per-branch typed (5 forms — mirror of sv_2017 with positional shift)

```ebnf
module_declaration_sv_2023 := @sample: "module m; endmodule" module_ansi_header (timeunits_declaration)? non_port_module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "ansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(a); endmodule" module_nonansi_header (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "nonansi", header: $1, timeunits: $2, items: $3, end_label: $5}
                            | @sample: "module m(.*); endmodule" attribute_instance* module_keyword (lifetime)? module_identifier lparen dot star rparen semi (timeunits_declaration)? module_item* kw_endmodule_2eb38ec9 (colon module_identifier)?
                                -> {kind: "wildcard", attributes: $1, keyword: $2, lifetime: $3, name: $4, timeunits: $10, items: $11, end_label: $13}
                            | @sample: "extern module m(a);" kw_extern_bf1ee311 module_nonansi_header
                                -> {kind: "extern_nonansi", header: $2}
                            | @sample: "extern module m;" kw_extern_bf1ee311 module_ansi_header
                                -> {kind: "extern_ansi", header: $2}
```

Same kind labels as sv_2017; only the wildcard branch differs in positional indices. sv_2023 uses `dot star` (2 separate tokens) where sv_2017 uses `dot_star` (1 combined token), shifting the wildcard branch's later positional refs: `timeunits` from `$9` → `$10`, `items` from `$10` → `$11`, `end_label` from `$12` → `$13`. Same kind discriminator and field names are exposed to consumers — the profile-shift is invisible in the typed AST.

### Empirical pre/post on `module m; endmodule\n` (sv_2017 profile)

```text
# Pre — body field of description-kind source_text_item.body was raw envelope:
"source_text": [
  {
    "kind": "description",
    "body": {
      "kind": "module_declaration",
      "body": [<module_declaration_sv_2017 raw envelope>]   // 5-element array
    }
  }
]

# Post — three layers of typed dispatch:
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
```

### Consumer dispatch unlocked at the module-declaration level

```rust
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        if desc["kind"] == "module_declaration" {
            let module = &desc["body"];
            match module["kind"].as_str().unwrap() {
                "ansi" => walk_ansi(&module["header"], &module["timeunits"],
                                    &module["items"], &module["end_label"]),
                "nonansi" => walk_nonansi(&module["header"], &module["timeunits"],
                                          &module["items"], &module["end_label"]),
                "wildcard" => walk_wildcard(&module["attributes"], &module["keyword"],
                                            &module["lifetime"], &module["name"],
                                            &module["timeunits"], &module["items"],
                                            &module["end_label"]),
                "extern_nonansi" => walk_extern_nonansi(&module["header"]),
                "extern_ansi"    => walk_extern_ansi(&module["header"]),
                other => panic!("unknown module_declaration kind: {}", other),
            }
        }
    }
}
```

### Annotation inventory

31 entries (was 20). New: 1 (attribute_instance) + 5 (module_declaration_sv_2017) + 5 (module_declaration_sv_2023) = 11 added.

### `comment_only_source_region` typing — DEFERRED, blocked by task #38

This batch attempted to also type `comment_only_source_region := white_space* ( line_comment | block_comment ) ( white_space | line_comment | block_comment )*` with `-> {first: $2, rest: $3}`. Annotation didn't register: parser inventory count stayed unchanged after that change. This is task #38 (parens-grouped-Or trailing-annotation attribution bug — same class as the regex parser PGEN-EBNF gotcha logged earlier). The `comment_only_source_region` rule's two parens-grouped Or expressions cause the trailing `-> ...` annotation to attach to one of the inner Ors instead of the rule. Annotation reverted; sub-rule typing of comment_only_source_region is gated on task #38's resolution OR a grammar refactor that flattens the parens-grouped Ors into named helper rules.

### Same accept set, same diagnostic codes

Only the AST shape changed. No grammar accept-set or diagnostic-code change.

### Schema-version stays `1` (additive across all three slices).

### mdBook

`changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.

### Public API surface unchanged.

### Annotation-language idiom notes

- **`{first: $N, rest: $M}` workaround for parens-grouped quantified repetition** is a clean fallback when `[$N, $M**]` mixed-array spread isn't available. Used here on attribute_instance.
- **Multi-line per-branch annotation with `@sample:` metadata** preserved correctly through the codegen path. PGEN's EBNF parser treats `@sample: "..."` as branch metadata that doesn't shift positional indices for the `-> ...` annotation following the branch body.

### Next slice candidates

- Type `module_ansi_header` per-branch (currently the unwalked `header:` field on the ansi/extern_ansi forms).
- Type `module_nonansi_header` per-branch.
- Type `module_keyword` (2-form Or: `module` / `macromodule`).
- Type `interface_declaration`, `package_declaration`, `class_declaration` per-branch (sibling rules to module_declaration).
- Address task #38 to unblock comment_only_source_region + similar parens-grouped-Or rules.

## Release 1.0.5 / Contract 1.0.5 Highlights — SV-Slice-5: `compiler_directive` transparent passthrough (clean directive text)

- **Annotation:** `compiler_directive := trivia /` `` `[^\r\n]*/`` `` -> $2` (line 226 of `grammars/systemverilog.ebnf`).
- **Effect:** drops the leading `trivia` (whitespace) prefix from the matched sequence and emits just the captured directive text (the `` ` `` backtick + directive name + arguments) as a clean JSON string. When `source_text_item.kind == "compiler_directive"`, the `body` field is now a directly-usable string instead of a nested envelope.
- **Empirical pre/post on `` `define FOO bar `` followed by `module m; endmodule\n`:**

```text
# Pre-SV-Slice-5 — body was the raw envelope of `trivia regex_capture`:
"source_text": [
  {
    "kind": "compiler_directive",
    "body": [<trivia envelope>, "`define FOO bar"]   // 2-element array
  },
  {"kind": "description", "body": {...}}
]

# Post-SV-Slice-5 — body is the clean directive string:
"source_text": [
  {
    "kind": "compiler_directive",
    "body": "`define FOO bar"   // clean string, ready to use
  },
  {"kind": "description", "body": {...}}
]
```

- **Consumer dispatch is now trivially simple for compiler directives:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "compiler_directive" => {
            let directive_text = item["body"].as_str().unwrap();
            // directive_text is e.g. "`define FOO bar" — ready to feed to a
            // compiler-directive parser without further AST descent.
            process_directive(directive_text);
        }
        "description" => walk_description(&item["body"]),  // typed object
        // ... other kinds
    }
}
```

- **Annotation inventory:** 20 entries (was 19). New: `compiler_directive`. Existing: source_text (1), source_text_item (8), description (8), systemverilog_file (1), systemverilog_parseable_file (1).
- **Same accept set, same diagnostic codes.** Only the `compiler_directive` shape changed.
- **Schema-version stays `1`** (additive — clean string replaces a 2-element array; consumers walking with the dual-shape pattern handle both).
- **Heterogeneous body types per `kind`** are now in the SV AST: when `source_text_item.kind == "description"`, body is a typed object; when `kind == "compiler_directive"`, body is a string. Consumers dispatch on `kind` first, then handle the body shape per its type. This is the same pattern the regex AST uses (e.g. `atom.kind == "literal"` → body is string vs `atom.kind == "char_class"` → body is a typed object).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: transparent passthrough `-> $N` (no object literal) is the cleanest form for "extract just the captured payload" — used here on a 2-element sequence to drop the trivia prefix and surface only the regex match. Same idiom as regex.ebnf's `escape = "\\\\" escape_unit -> $2` (drops the leading backslash and surfaces the typed escape unit).

## Release 1.0.4 / Contract 1.0.4 Highlights — SV-Slice-4: `description` per-branch typed (`kind:` discriminator on 8 branches; attribute_instance* preserved)

- **Annotation:** 8 per-branch annotations on `description` (line 957 of `grammars/systemverilog.ebnf`):

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

- **Multi-element branches preserve attributes**: branches 6 and 7 (`attribute_instance* package_item` / `attribute_instance* bind_directive`) carry the `attribute_instance*` prefix as a separate `attributes:` field while keeping the inner construct as `body:`. Consumers can walk attributes and body independently. The other 6 branches are single-element and use the simpler `{kind, body}` shape.
- **Effect:** items in `systemverilog_file.source_text` now carry **two layers of typed dispatch end-to-end**:
  - Outer `source_text_item.kind` (from SV-Slice-3) — identifies which top-level slot the item came from.
  - Inner `description.kind` (this slice) — when the outer kind is `"description"`, identifies which specific construct (module/interface/class/etc.).
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-4 — source_text[0].body was the raw description envelope:
"source_text": [
  {
    "kind": "description",
    "body": [<description Or-of-8 raw envelope, with module_declaration matched in branch 0>]
  }
]

# Post-SV-Slice-4 — source_text[0].body carries its own typed kind:
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

- **Consumer dispatch unlocked at the description level:**

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

- **Annotation inventory:** 19 entries (was 11). 8 new per-branch entries on `description`. Existing: source_text (1), source_text_item (8), systemverilog_file (1), systemverilog_parseable_file (1).
- **Same accept set, same diagnostic codes.** Only the `description` shape changed.
- **Inner `module_declaration`, `udp_declaration`, etc. shapes still raw envelope** — per-rule typing of those is a follow-up slice. The `description.kind` discriminator gives consumers the dispatch hook to route to per-construct walkers.
- **Schema-version stays `1`** (additive — discriminator on existing branches).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: multi-element branch `{kind: "<name>", attributes: $1, body: $2}` is a clean preservation of leading-quantified-prefix semantics — same idiom would apply to any rule whose branch has `<quantified_prefix>* <main_body>` shape (very common in SV grammar around attribute decorations).

## Release 1.0.3 / Contract 1.0.3 Highlights — SV-Slice-3: `source_text_item` per-branch typed (`kind:` discriminator on 8 branches)

- **Annotation:** 8 per-branch annotations on `source_text_item` (lines 210-217 of `grammars/systemverilog.ebnf`):

```ebnf
source_text_item := description                       -> {kind: "description", body: $1}
                  | local_parameter_declaration semi  -> {kind: "local_parameter_declaration", body: $1}
                  | parameter_declaration semi        -> {kind: "parameter_declaration", body: $1}
                  | package_import_declaration         -> {kind: "package_import_declaration", body: $1}
                  | timeunits_declaration              -> {kind: "timeunits_declaration", body: $1}
                  | compiler_directive                 -> {kind: "compiler_directive", body: $1}
                  | comment_only_source_region         -> {kind: "comment_only_source_region", body: $1}
                  | semi                               -> {kind: "semi"}
```

- **Effect:** every item in the `systemverilog_file.source_text` array now carries an explicit `kind:` discriminator. Consumers walking `obj["source_text"]` can dispatch on `item["kind"]` instead of structural recursion to identify which top-level construct each item is.
- **`semi` branch carries no body** (it's just a stray `;` — no useful payload). The other 7 branches carry the matched sub-rule's raw envelope as `body`.
- **`local_parameter_declaration semi` and `parameter_declaration semi` branches drop the trailing `semi`** (annotation references `$1` only, not `$2`). The semicolon is a syntactic delimiter, not part of the typed shape.
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-3 — source_text[0] was the matched-branch shape directly:
"source_text": [
  [<description envelope — module_declaration shape>]
]

# Post-SV-Slice-3 — source_text[0] is a typed object with discriminator:
"source_text": [
  {
    "kind": "description",
    "body": [<description envelope — module_declaration shape>]
  }
]
```

- **Annotation inventory:** 11 entries (was 3). New: 8 per-branch entries on `source_text_item`. Existing: `source_text`, `systemverilog_file`, `systemverilog_parseable_file`.
- **Same accept set, same diagnostic codes.** Only the `source_text_item` shape changed.
- **`@branch_policy: priority_first` and `@priority: [24, 16, 16, 12, 10, 8, 6, 4]` preserved** — the branch-policy / priority metadata applies before annotation dispatch, no change needed.
- **Inner `description`, `local_parameter_declaration`, etc. shapes still raw envelope** — per-rule typing of those rules is a follow-up slice. The `kind:` discriminator gives consumers the dispatch hook to route to per-branch walkers; the walkers themselves currently descend the raw envelope.
- **Schema-version stays `1`** (additive — discriminator on existing branches, no new rules or accept-set change).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: per-branch `{kind: "<name>", body: $1}` is the canonical regex-campaign idiom for Or-rule per-branch typing (used in regex slices 7, 8, 14-21, etc.). Verified to work for SV grammar with metadata blocks (`@branch_policy`, `@priority`) preserved.

## Release 1.0.2 / Contract 1.0.2 Highlights — SV-Slice-2: `source_text` typed via `[$1**]` flatten-spread

- **Annotation:** `source_text := source_text_item* -> [$1**]` (line 2273 of `grammars/systemverilog.ebnf`).
- **Effect:** the `source_text` field of every typed `systemverilog_file` JSON object is now a **flat array** of `source_text_item` shapes. Pre-fix it carried the raw Quantified envelope wrapping the iteration — consumers walking `obj["source_text"]` had to descend through the Quantified wrap before reaching items. Post-fix the array is consumer-ready; iterate directly.
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-2 — source_text was a Quantified envelope:
{
  "type": "systemverilog_file",
  "source_text": [<Quantified-wrapped iteration of source_text_item>]
}

# Post-SV-Slice-2 — source_text is a flat array of items:
{
  "type": "systemverilog_file",
  "source_text": [<source_text_item shape>]   // length = 1 for minimal_module
}
```

- **`source_text_item` itself stays raw envelope** (Or of `description | local_parameter_declaration semi | parameter_declaration semi | package_import_declaration | bind_directive | ...`). Per-branch typing of source_text_item is a follow-up slice; this slice only flattens the parent.
- **Annotation inventory:** 3 entries (was 2). New: `source_text`. Existing: `systemverilog_file`, `systemverilog_parseable_file`.
- **Same accept set, same diagnostic codes.** Only the `source_text` array shape changed.
- **Same `expected_json_object_keys_present` and `expected_json_object_string_values`** in the manifest's `minimal_module` sample. The rule-under-test is `systemverilog_file`, whose top-level keys (`type`, `source_text`) and `type` value (`"systemverilog_file"`) are unchanged. The Slice-2 change is in the SHAPE of the `source_text` value, not its key presence — manifest's drift-status updated to `calibrated_2026_05_04_slice_2` to record the calibration.
- **Schema-version stays `1`** (additive — flat-array shape is strictly a clean-up of the raw envelope, no new keys or rules).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: `[$1**]` is the canonical regex-campaign idiom for "flatten an array-shaped sub-rule reference into the enclosing array literal" — same idiom used in regex.ebnf for `concatenation = piece+ -> [$1**]` (slice 1 of the regex campaign). Verified to work for SV grammar's first array-shaped rule.

## Release 1.0.1 / Contract 1.0.1 Highlights — SV-Slice-1: `systemverilog_file` typed (dangling annotation rescued)

- **First effective annotation on the systemverilog parser.** Pre-fix `grammars/systemverilog.ebnf` carried two intended annotations that were both broken:
  1. Line 195's `-> {type: "systemverilog_file", source_text: $2}` was **dangling** — separated from its intended rule `systemverilog_file` (line 184) by a blank line + the `sv_multi_entry_root` helper rule (line 193) + another blank line. The annotation latched onto nothing effective; the parser registered 0 annotations for `systemverilog_file` and the rule's AST output stayed as the recursive-envelope `Sequence` shape.
  2. Line 200's `// -> {type: "systemverilog_parseable_file", items: $2}` had a `//` prefix (presumed by the grammar author to be a comment) but PGEN's EBNF dialect uses `#` for comments, not `//` — the `// ` was treated as literal noise and the rest of the line was parsed as a real annotation. So the `systemverilog_parseable_file` annotation was actually registered, but accidentally so.
- **Fix:** moved the dangling line-195 annotation up onto `systemverilog_file := trivia source_text trivia` (line 184) using the canonical multi-line continuation form. Removed the misleading `//` prefix from the line-200 annotation since it was effectively active anyway. Both annotations now register through the documented path.

```ebnf
# After SV-Slice-1:
systemverilog_file := trivia source_text trivia
                   -> {type: "systemverilog_file", source_text: $2}
...
systemverilog_parseable_file := trivia parseable_source_item* trivia
                             -> {type: "systemverilog_parseable_file", items: $2}
```

- **Empirical verification.** Generated the parser via `ast_pipeline grammars/systemverilog.ebnf --generate-parser --eliminate-left-recursion`, built `parseability_probe` with the `PGEN_SYSTEMVERILOG_PARSER_PATH` adapter, ran it on `module m; endmodule\n` with `--profile sv_2017`. AST root pre-fix: `{"content": {"Sequence": [...]}}` (recursive envelope). Post-fix: `{"content": {"Json": {"type": "systemverilog_file", "source_text": [...]}}}`. The annotation correctly latches and the typed shape lands.
- **Annotation inventory** (from `ast_pipeline`'s reporting): 2 entries — `systemverilog_file` and `systemverilog_parseable_file`. Was 1 entry pre-fix (only the accidentally-registered `systemverilog_parseable_file`).
- **Manifest update.** `rust/test_data/ast_shape_contract/systemverilog_v1.json` `current_content_kind` updated from placeholder `"sequence"` to calibrated `"json_object"`. `drift_status` flipped from `parser_unavailable_in_default_build_pending_first_run_calibration` to `calibrated_2026_05_04`. Layout note about line 195 dangling annotation removed (resolved by this slice). Calibration history added.
- **Annotation campaign starts here.** This is SV-Slice-1 — the first slice in the systematic return-annotation campaign on `grammars/systemverilog.ebnf`, mirroring the regex parser's 42-slice campaign. Subsequent slices will type rules one-by-one (`description`, `module_declaration`, `interface_declaration`, etc.). Each slice bumps parser release / contract version and appends a Highlights section here.
- **No accept-set change.** The grammar's accept set is unchanged — same inputs parse, same inputs reject. Only the AST shape for accepted inputs changes (recursive envelope → typed `{type, source_text}` object at the root).
- **Schema-version stays `1`.** Per the schema versioning policy, additive shape changes within a major version don't bump the schema number; the `current_content_kind` change is tracked in the per-rule manifest.
- **mdBook**: `docs/systemverilog_parser_book/src/changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, and `rules-top-level.md` updated to reflect the typed shape. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- No SV-NNNN bug ledger entry (this is a foundation-slice annotation correctness fix, not a downstream-reported bug).

## Release 1.0.0 / Contract 1.0.0 Highlights — initial baseline (foundation deliverables landed)

- **Initial contract identity baseline.** The systemverilog parser has been part of PGEN for some time; this contract document is being upgraded from a thin "stable surface" pointer into the same release-tracked Highlights structure used by `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`. Subsequent return-annotation slices on `grammars/systemverilog.ebnf` will each bump the parser-release / contract-version pair and append a Highlights section here.
- **mdBook scaffolded.** `docs/systemverilog_parser_book/` is the new canonical AST reference for downstream consumers. Initial chapters: welcome, build recipe, public API, AST envelope, schema versioning, changelog index, glossary. Per-rule and per-example chapters land as the annotation campaign progresses.
- **Build status.** The generated systemverilog parser is **NOT in the default `cargo test --features generated_parsers` build**. It is produced on-demand by `sv_stimuli_quality_gate` (and similar gates) into `rust/target/<gate>/work/systemverilog_parser.rs` and discarded after the gate run. Cfg `has_generated_systemverilog_parser` therefore stays off in default builds. When the parser is present (gate run or `PGEN_SYSTEMVERILOG_PARSER_PATH` override), the embedding API path is enabled and the per-family AST-shape contract test activates.
- **Schema baseline.** `1` — corresponds to the `version: 1` field in `rust/test_data/ast_shape_contract/systemverilog_v1.json`. The manifest currently carries one stub sample (`minimal_module: "module m; endmodule\n"`) calibrated against the placeholder `current_content_kind: "sequence"`. First post-foundation slice will run the parser, observe the actual content kind, and either confirm or update the manifest.
- **Annotation campaign — not yet started.** `grammars/systemverilog.ebnf` is currently un-annotated except for one commented-out trial annotation at line 200. Subsequent slices will systematically apply return annotations rule-by-rule, mirroring the regex parser campaign that produced typed shapes for 42+ regex rules. Schema-version bumps and contract Highlights sections will track each slice.
- **Public API surface unchanged.** No accept-set or diagnostic-code change in this baseline.
- **Bug ledger entries**: any released SV parser bug is tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` under the `SV-NNNN` ID family. None blocking the baseline.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog.ebnf`
  - Companion (LRM-extracted reference): `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`
  - Companion (profiled wrappers): `grammars/systemverilog_lrm_profiled_generated.ebnf`, `grammars/systemverilog_lrm_profiled_wrapper.ebnf`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- AST shape contract manifest:
  - `rust/test_data/ast_shape_contract/systemverilog_v1.json`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PARSER_PATH`
- Live closure/status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`
- Reference IEEE 1800 LRM corpus (read-only):
  - `docs/systemverilog/2017/` (Annex A formal syntax, plus other annexes)
  - `docs/systemverilog/2023/` (delta + 2023-specific annexes)

## Stable Integration Surface
- Grammar family:
  - `systemverilog`
- Stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Stable convenience entry points:
  - `parse_systemverilog_2017(...)`
  - `parse_systemverilog_2023(...)`
  - `parse_systemverilog_2017_ast_dump(...)`
  - `parse_systemverilog_2023_ast_dump(...)`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_named(...)` (string-name overload)
  - `parse_grammar_profile_named_with_limits(...)` (string-name overload with explicit limits)
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Build / Availability Requirements
- Downstream consumers should treat the generated backend as required for real host integration.
- Startup should inspect `parser_embedding_api_contract().supports_systemverilog_generated_backend`.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.
- When local validation sets `PGEN_SYSTEMVERILOG_PARSER_PATH` while invoking `cargo ... --manifest-path rust/Cargo.toml`, use an absolute path or a path relative to `rust/`; `rust/build.rs` resolves that variable relative to the Rust manifest directory.
- The PGEN-side `sv_stimuli_quality_gate` make target produces the generated parser at `rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs`. Downstream embedders that vendor this artifact should track its SHA256 against the parser-release version recorded in this contract.

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
  - `make -C rust SHELL=/bin/bash nexsim_parser_embedding_contract_gate`
- Family closure / proof:
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_parser_family_status_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_parser_family_status_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_combined_telemetry_contract_gate`
- Stimuli / corpus:
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_syntax_closure_gate`
- Documentation:
  - `make systemverilog_parser_book_gate` — builds the mdBook + verifies tracked HTML output.

## Scope / Non-Goals
- The stable contract is the host-oriented embedding surface in `pgen::embedding_api`, not internal generated parser types.
- Internal AST node types are not the downstream contract.
- The current tracked sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
