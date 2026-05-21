# PGEN Semantic Store — Schema Language Specification

**Status:** DRAFT (sketch — first cut for review).
**Spec version:** 0.1.0 (pre-1.0 — breaking changes allowed).
**Companion to:** `docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md`. This document specifies the *surface syntax* of the seven-stage lifecycle (`CONTEXT_AWARE_PARSING_DESIGN.md` §4); the API contract specifies its *semantics* and the engine-level Rust API.
**Target file:** `grammars/semantic_annotation.ebnf` (extensions land here in Phase 1; this spec defines what those extensions will look like).

---

## 1. Purpose

This spec defines, in precise EBNF, the surface syntax a grammar author writes to interact with the semantic store. Every form below corresponds 1:1 with a Stage in `CONTEXT_AWARE_PARSING_DESIGN.md` §4:

| Form | Stage |
|---|---|
| `@fact_kind: { ... }` | 1 — DECLARE |
| `@emit_fact: { ... }` | 2 — EMIT |
| `@predicate <name> args:[...] phase:<phase>` | 3 — QUERY (primitive) |
| `@predicate_def: { ... }` | 3 — QUERY (composed definition) |
| `@open_scope: { ... }` / `@close_scope` | 4 — SCOPE |
| (none; auto-derived from `exportable: true` in DECLARE) | 5 — EXPORT |
| `@import_from_library: { ... }` | 6 — IMPORT |
| (none; auto by engine, `.3.3.3` IIFE) | 7 — ROLLBACK |

## 2. Scope of additions to `semantic_annotation.ebnf`

The current `grammars/semantic_annotation.ebnf` defines a small set of directive headers (`@emit_fact`, `@predicate`, `@open_scope`, `@close_scope`, `@export_to_library`, `@import_from_library`). This spec **extends** it with:

- A new `@fact_kind:` declaration block (Stage 1 — does not exist today).
- A new `@predicate_def:` composed-predicate block (Stage 3 composed form — does not exist today).
- A formalised attribute-key list / index-tuple list / scope-kind-label vocabulary for `@fact_kind:`.
- A formalised predicate-body expression language for `@predicate_def:`.
- An obsoletion path for the current `@export_to_library` directive: replaced by the `exportable: true` field on the corresponding `@fact_kind:` declaration. The existing directive remains parseable for backward compatibility but is deprecated.

Nothing in this spec removes existing forms unless explicitly noted (deprecation > removal, per §10.2 of the API contract).

## 3. Reused productions (already in `semantic_annotation.ebnf`)

This spec reuses existing productions verbatim:

```ebnf
simple_identifier := /[a-zA-Z_][a-zA-Z0-9_]*/
string_literal    := "\"" /([^"\\]|\\.)*/ "\""
bool_literal      := "true" | "false"
integer_literal   := /[0-9]+/
rule_reference    := "$" rule_reference_name                   # $name, $1, $x.y, $x[0], dotted+indexed chains (per .3.3.4.a.1/.a.2)
```

Where new productions add fields specific to this spec, they are introduced in §4–§9 below.

## 4. `@fact_kind:` — DECLARE block

### 4.1 EBNF

```ebnf
fact_kind_declaration
    := "@fact_kind:" "{" fact_kind_field ( "," fact_kind_field )* ","? "}"

fact_kind_field
    := "name:"          simple_identifier
     | "attributes:"    attribute_list
     | "required:"      attribute_list
     | "indexes:"       index_list
     | "scope_kind:"    simple_identifier
     | "exportable:"    bool_literal
     | "artefact_kind:" simple_identifier
     | "description:"   string_literal

attribute_list  := "[" simple_identifier ( "," simple_identifier )* ","? "]"
index_list      := "[" index_tuple        ( "," index_tuple        )* ","? "]"
index_tuple     := "(" simple_identifier ( "," simple_identifier )* ","? ")"
```

### 4.2 Mandatory + optional fields

| Field | Required | Default | Notes |
|---|---|---|---|
| `name` | **yes** | — | snake_case, unique within the grammar |
| `attributes` | **yes** | — | non-empty list of attribute names |
| `required` | no | `[]` | subset of `attributes`; emit-time validation |
| `indexes` | no | `[(scope, kind, name)]` | each tuple is a composite key; engine maintains one index per tuple |
| `scope_kind` | no | `current` | which scope kind facts of this kind live in by default |
| `exportable` | no | `false` | eligibility for library export |
| `artefact_kind` | no | same as `name` | directory name under `<lib-dir>/<scope_kind>/` for exports |
| `description` | no | `""` | for documentation / `--explain` |

### 4.3 Validation rules (codegen-time)

V-DECL-1 — `name` is unique across all `@fact_kind:` blocks in this grammar.
V-DECL-2 — `attributes` is non-empty.
V-DECL-3 — every name in `required` appears in `attributes`.
V-DECL-4 — every name in every `index_tuple` appears in `attributes` ∪ `{scope, kind}` (the implicit attributes "scope" and "kind" are always indexable).
V-DECL-5 — `index_tuple` is non-empty and contains no duplicate attribute names.
V-DECL-6 — `scope_kind` (if specified) matches one of the labels used in this grammar's `@open_scope` directives, or one of the engine reserved labels (`global`, `current`, `enclosing_block`, `enclosing_function`, `enclosing_class`, `enclosing_package`, `enclosing_file`).
V-DECL-7 — `name` and `artefact_kind` (if specified) are valid path components (no `/`, no `..`, no leading dot).

Violation of any V-DECL rule = grammar-compile-time error with a precise message including the field, the offending value, and the rule name (e.g., "V-DECL-3: `required: [type_kindd]` but `type_kindd` is not in `attributes: [name, type_kind]`").

### 4.4 Example

```ebnf
@fact_kind: {
  name:           variable_binding,
  attributes:     [name, type_kind, type_ref, declared_in],
  required:       [name, type_kind],
  indexes:        [(scope, name), (scope, type_kind), (name)],
  scope_kind:     enclosing_block,
  exportable:     true,
  artefact_kind:  bindings,
  description:    "A bound identifier (var / param / port / field / local) with its declared type."
}
```

## 5. `@emit_fact:` — EMIT directive

### 5.1 EBNF (extends today's `@emit_fact`)

```ebnf
emit_fact_directive
    := "@emit_fact:" "{" emit_fact_field ( "," emit_fact_field )* ","? "}"

emit_fact_field
    := "kind:" simple_identifier
     | simple_identifier ":" value_expression

value_expression
    := rule_reference       # $name, $1, $x.y, $x[0], etc. (existing)
     | string_literal
     | integer_literal
     | bool_literal
     | "[" value_expression ( "," value_expression )* ","? "]"   # list literal
```

### 5.2 Position relative to the rule

The directive precedes the rule it annotates (same convention as today's `@emit_fact`):

```ebnf
@emit_fact: { kind: variable_binding, name: $1, type_kind: $2.kind }
variable_decl_assignment := simple_identifier type_descriptor ...
```

### 5.3 Validation rules

V-EMIT-1 — `kind` is mandatory; missing → codegen error.
V-EMIT-2 — `kind` references a declared `@fact_kind`; missing → codegen error.
V-EMIT-3 — every attribute key (other than `kind`) is in the kind's declared `attributes`; unknown → codegen error.
V-EMIT-4 — At runtime: every attribute in the kind's `required` has a value (after value-expression evaluation); missing → parse-time `EmitError::MissingRequired`. (Codegen cannot prove this since values are runtime, but codegen warns if no producing value-expression is present and the attribute is required.)

### 5.4 Example

```ebnf
@emit_fact: { kind: variable_binding,
              name: $variable_name,
              type_kind: $resolved_type_kind,
              type_ref: $type_descriptor }
variable_decl_assignment := ...
```

## 6. `@predicate <name> args:[...] phase:<phase>` — QUERY primitive

### 6.1 EBNF (extends today's `@predicate`)

```ebnf
predicate_directive
    := "@predicate" predicate_name "args:" predicate_args ( "phase:" predicate_phase )?

predicate_name
    := simple_identifier

predicate_args
    := "[" value_expression ( "," value_expression )* ","? "]"

predicate_phase
    := "pre" | "branch" | "post"
```

### 6.2 Built-in primitives

The engine provides four built-in predicate names; the grammar author may not redefine them:

| Predicate | Arg shape | Result | Phase usage |
|---|---|---|---|
| `has_fact` | `[<kind>, <name>]` | true iff a fact of `<kind>` with `<name>` exists in any visible scope | any |
| `fact_attribute_equals` | `[<kind>, <name>, <attr>, <value>]` | true iff `has_fact` and `<attr>` equals `<value>` | any |
| `fact_count_at_least` | `[<kind>, <M>]` | true iff ≥ M facts of `<kind>` exist | any |
| `resolve_path` | `[<dotted_name>]` | true iff the path resolves to a fact | branch / post |

Phase semantics:
- **`pre`** — checked **before** the rule body parses; fail = rule fails immediately.
- **`branch`** — checked **inside an ordered choice**, gating which branch fires; fail = this branch fails, next branch tried.
- **`post`** — checked **after** the rule body parses, before commit; fail = rule fails, transaction rolls back.

### 6.3 Validation rules

V-QPRIM-1 — `predicate_name` is one of the four built-ins, or matches a `@predicate_def:` defined elsewhere in this grammar.
V-QPRIM-2 — `args` arity and shape match the predicate's declared signature.
V-QPRIM-3 — `phase` if specified is one of `pre|branch|post`; default depends on rule position (TBD — codified in API contract §3.4).

### 6.4 Examples

```ebnf
@predicate has_fact args:[variable_binding, $1] phase: post
known_unscoped_variable_identifier := simple_identifier
```

```ebnf
@branch_policy: predicate_first
method_call_body :=
        @predicate receiver_is_array args:[$enclosing_receiver] phase: branch
        built_in_method_call                                                  -> {kind: "built_in", body: $1}
    |   method_identifier attribute_instance* lparen list_of_arguments rparen -> {...}
```

## 7. `@predicate_def:` — composed predicate definition

### 7.1 EBNF

```ebnf
predicate_def_block
    := "@predicate_def:" "{" predicate_def_field ( "," predicate_def_field )* ","? "}"

predicate_def_field
    := "name:" simple_identifier
     | "args:" "[" simple_identifier ( "," simple_identifier )* ","? "]"
     | "body:" predicate_expression

predicate_expression
    := predicate_term ( predicate_logical_op predicate_term )*

predicate_term
    := primitive_call
     | "(" predicate_expression ")"
     | "!" predicate_term
     | predicate_path_access

primitive_call
    := built_in_predicate_name "(" predicate_arg_list ")"

built_in_predicate_name
    := "has_fact" | "fact_attribute_equals" | "fact_count_at_least" | "resolve_path"

predicate_arg_list
    := value_expression ( "," value_expression )*

predicate_path_access
    := primitive_call "." "attribute" "(" string_literal ")"

predicate_logical_op
    := "&&" | "||"

# Membership operator on attribute values:
predicate_term := predicate_path_access "in" "[" value_expression ( "," value_expression )* ","? "]"
                | predicate_path_access predicate_compare_op value_expression
predicate_compare_op
    := "==" | "!=" | "<" | "<=" | ">" | ">="
```

### 7.2 Validation rules

V-QDEF-1 — `name` is unique across `@predicate_def:` blocks in this grammar; does not shadow a built-in.
V-QDEF-2 — `args` lists distinct identifiers.
V-QDEF-3 — every primitive call in `body` uses a built-in predicate name with correct arity.
V-QDEF-4 — every `$arg` reference in `body` refers to a name in `args`.
V-QDEF-5 — `predicate_path_access` is only applied to primitives that return a fact reference (`resolve_path`); applying it to `has_fact` (which returns bool) is a codegen error.

### 7.3 Example

```ebnf
@predicate_def: {
  name: receiver_is_array,
  args: [receiver_path],
  body: resolve_path($receiver_path).attribute("type_kind") in ["array", "queue", "dynamic_array", "assoc_array"]
}
```

then used as a primitive call:

```ebnf
@predicate receiver_is_array args:[$1] phase: branch
some_rule := ...
```

## 8. `@open_scope:` / `@close_scope` — SCOPE directives

### 8.1 EBNF (extends today's directives)

```ebnf
open_scope_directive
    := "@open_scope:" "{" "kind:" simple_identifier "," "name:" value_expression ","? "}"

close_scope_directive
    := "@close_scope"
```

### 8.2 Validation rules

V-SCOPE-1 — `kind` is non-empty.
V-SCOPE-2 — `@close_scope` is only valid on a rule whose enclosing-rule chain opened a scope earlier (codegen tracks the per-rule scope-open/close balance; mismatched → error).
V-SCOPE-3 — `kind` labels used in `@open_scope` must be a subset of the labels referenced in any `@fact_kind: { scope_kind: ... }` field (codegen warns on unused labels).

### 8.3 Example

```ebnf
@open_scope: { kind: class, name: $1 }
class_declaration := simple_identifier ... @close_scope
```

## 9. `@import_from_library:` — IMPORT directive

### 9.1 EBNF (extends today's directive)

```ebnf
import_from_library_directive
    := "@import_from_library:" "{" "kind:" simple_identifier "," "name:" value_expression ","? "}"
```

### 9.2 Validation rules

V-IMPORT-1 — `kind` matches the `artefact_kind` of some declared `@fact_kind` in this or another grammar (codegen warns if not declared in this grammar; runtime error if the artefact's kind doesn't match anything).

### 9.3 Example

```ebnf
@import_from_library: { kind: bindings, name: $package_name }
package_import_item := kw_import package_identifier "::" identifier
```

## 10. Deprecation: `@export_to_library`

The current `@export_to_library` directive (introduced in `.3.3.4.a` MVP-0) is **deprecated** in favour of the `exportable: true` field on the corresponding `@fact_kind` declaration. Migration:

```ebnf
# Before (deprecated):
@export_to_library: { kind: package, name_from: $body }
package_declaration := ...

# After:
@fact_kind: {
  name:        package_definition,
  attributes:  [name, source_file],
  required:    [name],
  indexes:     [(name)],
  scope_kind:  package,
  exportable:  true,
  artefact_kind: package_definitions
}
@emit_fact: { kind: package_definition, name: $name, source_file: $current_file }
package_declaration := ...
```

The deprecated form remains parseable for one major version after introduction of `@fact_kind:` (per API contract §2 stability rules); after that it is removed.

## 11. Worked example — all forms in context

Tiny SV grammar fragment exercising every stage:

```ebnf
# Stage 1 — declare two fact-kinds.
@fact_kind: {
  name: type_binding,
  attributes: [name, kind],
  required: [name, kind],
  indexes: [(scope, name), (name)],
  scope_kind: enclosing_package,
  exportable: true,
  artefact_kind: types,
  description: "A typedef-bound or class-bound type name."
}

@fact_kind: {
  name: variable_binding,
  attributes: [name, type_kind, type_ref],
  required: [name, type_kind],
  indexes: [(scope, name), (scope, type_kind)],
  scope_kind: enclosing_block,
  exportable: false
}

# Stage 3 — define a composed predicate.
@predicate_def: {
  name: receiver_is_array,
  args: [receiver_path],
  body: resolve_path($receiver_path).attribute("type_kind") in ["array", "queue", "dynamic_array", "assoc_array"]
}

# Stage 4 — open a package scope.
@open_scope: { kind: package, name: $1 }
package_declaration := kw_package simple_identifier ";" package_body kw_endpackage @close_scope

# Stage 2 — emit on a typedef.
@emit_fact: { kind: type_binding, name: $declared_name, kind: $declared_type_kind }
typedef_declaration := kw_typedef type_descriptor declared_identifier ";"

# Stage 2 — emit on a variable decl.
@emit_fact: { kind: variable_binding, name: $variable_name, type_kind: $resolved_kind, type_ref: $type_ref }
variable_decl_assignment := type_descriptor declared_identifier ( "=" expression )? ";"

# Stage 6 — import another package's facts.
@import_from_library: { kind: types, name: $package_name }
package_import_item := kw_import package_identifier "::" identifier

# Stage 3 — use the composed predicate to gate a branch.
@branch_policy: predicate_first
method_call_body :=
        @predicate receiver_is_array args:[$enclosing_receiver] phase: branch
        built_in_method_call                                                  -> {kind: "built_in", body: $1}
    |   method_identifier attribute_instance* lparen list_of_arguments rparen -> {kind: "call_with_args", ...}
```

Stages 5 (EXPORT) and 7 (ROLLBACK) have no surface syntax — they are derived from `exportable: true` (Stage 1) and the IIFE transaction boundary (engine-internal) respectively.

## 12. Open questions (for review)

- **`[TBC]`** Trailing commas inside `[...]` and `{...}` — accept (Rust-style) or reject (JSON-style)? Sketch currently accepts. Lean: accept.
- **`[TBC]`** Should `predicate_def.body` support `if-then-else` expressions, or only short-circuit `&&` / `||`? Sketch limits to boolean combinators. Adding `if-then-else` would let predicates return non-boolean values, which complicates the composable-predicate model. Lean: keep boolean-only.
- **`[TBC]`** Should `value_expression` in `@emit_fact:` allow inline arithmetic / string concatenation? Currently no — values come from rule captures or literals only. Lean: keep restrictive; complex value derivation belongs in the grammar, not the directive.
- **`[TBC]`** Should `@predicate_def:` allow recursion (one predicate calling another defined predicate)? Sketch is silent; codegen would either inline or maintain a call graph. Lean: allow, with cycle detection at codegen.
- **`[TBC]`** Naming conventions: do we enforce snake_case for `name`, `kind`, attribute names? Sketch is silent (any `simple_identifier` works). Lean: enforce snake_case via a warning, not a hard error.

## 13. References

- `docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md` — design.
- `docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md` — semantics + Rust API for the surface defined here.
- `grammars/semantic_annotation.ebnf` — target file for Phase 1 EBNF extensions.
- `grammars/return_annotation.ebnf` — sibling annotation language for return-shape directives (not extended here).
- `docs/RETURN_ANNOTATIONS_REFERENCE.md` / `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` — current annotation-language docs; will be cross-linked once this spec lands.

---

**Amend freely.** When approved, this spec becomes the binding surface that artefact 3 (test plan) writes test cases against and that Phase 1 EBNF edits to `grammars/semantic_annotation.ebnf` must implement faithfully.
