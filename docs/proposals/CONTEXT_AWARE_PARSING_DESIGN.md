# Context-Aware Parsing via Semantic Annotations

**Status:** DRAFT — for review and amendment. No code change attached.
**Owner:** PGEN engine + SystemVerilog grammar.
**Authors:** discussion 2026-05-21, after `.3.3.4.b.4` diagnosis (uvm_pkg `if(a.b.c(x))` failure).
**Restore tag for the pre-design baseline:** `checkpoint/post-3-3-4-b-3-layer-0-clean-pre-next-lrm-defect` @ `4195ee22`.

> This proposal is intentionally *amendable*. Anywhere you see **`[TBC]`** or **`Alternative:`**, that section is up for revision. The architectural principle in §1 is the part we want to land first; the mechanism in §3–§7 follows from it.

---

## 1. Principle

The parser is **a process with memory**. Semantic annotations are how that memory is built, organised, scoped, and queried. Their original role — *steering the parser generator at codegen time* — and their newer role — *steering the parser at parse time* — are converging into a single role: **"things we've seen and decided are with us wherever we go, and they help us decide what to do next."**

Consequence: when an EBNF rule has two or more syntactically-overlapping alternatives that the LRM cannot disambiguate by tokens alone (because the LRM grammar is intentionally permissive), the disambiguation comes from **prior context** — not from a syntactic patch, not from a lookahead trick, not from a "best-guess" priority order.

We therefore reject any "fallback path" that triggers when context is missing. If we have the right semantic at the right time, we always know which branch to take. Missing context = a bug to fix at the producer/scope side, not a fallback to take at the consumer side.

## 2. The concrete problem the design must solve

The immediate driver (and an excellent stress test) is the residual uvm/uvm_compat blocker.

**Minimal failing repro:**
```sv
package p;
function int f();
  if (seed_map.seed_table.exists(type_id)) begin
  end
endfunction
endpackage
```

The parser has no idea what `seed_map` is. Depending on the answer, the correct route for `seed_table.exists(type_id)` is one of:

| If `seed_map` is | then `seed_map.seed_table` is | then `.exists(type_id)` is | route |
|---|---|---|---|
| a class instance with associative-array field `seed_table` | array reference | array-method call | `built_in_method_call` |
| a class instance with method `seed_table` returning an object | method-call returning object | regular method call | `method_identifier (args)` |
| a hierarchical name path | a sub-hierarchical name | a free function call | `tf_call_with_args` or `system_tf_call` |

The LRM-extracted grammar has all three alternatives. Today the parser commits to whichever syntactically wins first — that's the drunk-stumbling failure mode.

**Goal:** the parser knows, at the moment it must choose a branch, what `seed_map` is, what `seed_map.seed_table` is, and therefore which alternative is correct.

## 3. Information the parser must carry

The model is **universal, not category-restricted**: any information encountered during parsing should be storable, and any later query should be efficiently answerable. The grammar must not have to pre-declare a fixed taxonomy of "interesting" facts — the engine's job is to organise whatever is emitted so retrieval is cheap regardless of the query shape.

### 3.1 Universal store, not curated categories

Concretely, the semantic store is a **general-purpose multi-indexed knowledge base** that the parser populates as it goes. Every `@emit_fact { kind: K, name: N, <attribute_K=V>* }` deposits a record. Queries are arbitrary combinations of those keys (`kind`, `name`, `scope`, attributes, position, ...). The same primitive that today supports `kind: type_name` will tomorrow support `kind: macro_define`, `kind: virtual_method_override`, `kind: constraint_block_membership`, `kind: covergroup_bin`, `kind: assertion_clock`, or anything else a grammar author finds useful — *without engine changes*. The engine does not know what categories exist; it just knows how to organise records by their declared keys.

### 3.2 Organisation techniques (the "various techniques" mandate)

Different queries have different shapes. The store maintains several indexes simultaneously, each optimised for one query family:

- **By `(scope, kind, name)`** — answers "does name N of kind K exist in this scope?" in O(1). The everyday lookup.
- **By `(scope, kind)`** — answers "list all facts of kind K in this scope" in O(out-size). Useful for iteration (e.g., "all variable bindings in this function").
- **By `kind`** — answers "list all facts of this kind across all scopes". Useful for global queries (e.g., "all type bindings", "all packages exported from this file").
- **By `(name, kind?)`** — answers "find all facts named N, optionally of kind K, walking the active scope chain outermost-first". The dotted-path resolver builds on this.
- **By `position`** — answers "what was the parse-time provenance of this fact?". Useful for diagnostics and consumer-side `@predicate` calls that reference `$position`.
- **By **attribute** — for kinds whose facts carry rich payload (e.g., `class_member` has a `container`), a secondary index keyed on the attribute name + value answers "all members of class C" in O(out-size).

The set of indexes is **declared per fact-kind**, not hard-coded in the engine — when a grammar emits a new kind, an optional accompanying schema describes which indexes to maintain for it (default: scope+name+kind). This keeps the engine schema-agnostic but lets grammar authors trade memory for query speed.

### 3.3 Scope organisation

Storage is **scope-aware** — every record lives in the scope active at emission time, and queries walk the scope chain from innermost to outermost (with explicit overrides for global / library-loaded facts).

- Scopes form a tree: file → package → class → method → block (and `generate` / `interface` / `modport` / `covergroup` / `constraint_block` / etc. as the grammar requires).
- A scope's records are visible to its descendants by default.
- An explicit `@open_scope kind: K, name: N` annotation opens a new scope; `@close_scope` closes it; nesting is the natural call-stack-like model.
- Cross-scope facts can be imported via `@import_from_library` (already in `.3.3.4.a` for cross-file imports — the same primitive extends naturally to intra-file scope sharing such as class-member access via dotted paths).

### 3.4 Query layer

Queries are themselves `@predicate` annotations on grammar rules — but the engine exposes a fixed set of query primitives that all grammar predicates compose. The minimum useful set:

- `exists(kind, name [, attributes])` — does a fact match? (today's `has_fact`).
- `attribute_of(kind, name) → value` — read an attribute (today's `fact_attribute_equals` and related).
- `resolve_path(dotted_name) → fact-or-unresolved` — multi-segment dotted resolution through the scope chain (this is the `seed_map.seed_table.exists` case from §2).
- `count(kind [, scope_filter])` — for cardinality predicates (already exists as `fact_count_at_least`).

Grammar predicates compose these. The engine guarantees each primitive is at-most logarithmic in store size (and typically constant).

### 3.5 Performance + scalability

The semantic store **shall by no means be the bottleneck of the parser**. It must support hundreds of thousands of facts (uvm-pkg alone emits tens of thousands of class members and a comparable number of variable bindings, before counting macro definitions, typedefs, covergroup bins, constraint memberships, etc.) while keeping every primitive operation under a tight latency budget.

Mandated properties:

- **Insertion budget**: O(number of indexes maintained for the fact-kind) per `@emit_fact`. With the typical 1–4 indexes per kind, that's effectively constant.
- **Lookup budget**: O(1) average for `(scope, kind, name)` and similar hash-indexed queries; O(log n) worst-case for any range / prefix index; O(out-size) for iteration queries.
- **Rollback budget**: O(emitted-in-tx) — never O(store). The `.3.3.3` IIFE pattern gives us the transaction boundary; every index maintains an undo log scoped to that boundary.
- **Library import budget**: lazy, on-demand. Importing a 50 MB compiled library must not stall the parser; only the cross-referenced subset is touched.
- **Memory layout**: arena-allocated scope nodes (cheap nested creation/destruction during parsing); facts are reference-counted or arena-owned, never globally allocated; no per-fact `Box` allocations on the hot path.
- **No quadratic anywhere on the parser's hot path**. Every query and every rollback must scale linearly with what it does, not with what's already in the store.

### 3.6 Schema definition language for fact-kinds

The principle of §3.1 (engine never enumerates kinds) needs a concrete surface so grammar authors can declare new fact-kinds and their indexes **at near-zero cost in dev effort**. The proposed syntax — to be refined in Phase 1 — is a top-of-grammar declaration block:

```ebnf
@fact_kind: {
  name:        variable_binding,
  attributes:  [name, type_kind, type_ref, scope],
  required:    [name, type_kind],            # validated at emit time
  indexes:     [(scope, name), (scope, type_kind)],
  exportable:  true,                          # eligible for @export_to_library
  description: "A bound identifier (var/param/field/local) with its declared type."
}

@fact_kind: {
  name:        class_member,
  attributes:  [container, name, member_kind, type_kind, type_ref, visibility, line],
  required:    [container, name, member_kind],
  indexes:     [(container, name), (container, member_kind), (name)],
  exportable:  true
}

@fact_kind: {
  name:        macro_define,
  attributes:  [name, params, body, source_file, line],
  required:    [name],
  indexes:     [(name)],
  exportable:  true
}
```

A grammar author who wants a new kind for an entirely new concept (assertion clocks, covergroup bins, constraint memberships, whatever) writes one declaration block. The engine:

- Allocates the declared indexes at codegen-or-load time.
- Validates `required` attributes on every `@emit_fact` of that kind.
- Auto-wires rollback (one undo log per index, scoped to the IIFE transaction boundary).
- Auto-wires library import/export for `exportable: true` kinds (using the `.3.3.4.a` artifact mechanism).
- Auto-generates query helpers (`has_fact`, `attribute_of`, etc.) parameterised on the declared attributes.

Adding a fact-kind is **a declaration, not an engine change**. That's the "won't cost us anything to describe this new semantic info type" budget the design must hit.

This is the "memory" §1 refers to — universal in what it stores, organised in many ways simultaneously for efficient retrieval, scoped + transactional + scalable by construction, and **extensible with zero engine churn**.

## 4. Lifecycle protocol — standardised operations for any semantic type

User mandate, 2026-05-21:

> "We need a standard, very efficient and systematic way of describing a new semantic fact to be added to the store, adding an instance of this new type to the store and retrieving such instance from the store or globally — and interactions with the store related to a semantic-annotation type (static) and life (runtime, parse time, collection, addition, retrieval) shall be systematised rigorously, no place for improvisation."
> "It shall be flexible and extensible so that it can deal with any semantic type we can think of."

**These two requirements compose:** the protocol below is the single, prescribed way EVERY semantic-fact operation happens, **and** it is built to accommodate any conceivable fact-kind without adding new mechanisms. Adding a new kind always means following the same protocol — never inventing a new pattern. The protocol's slot in §3.6's schema-declaration language is the only extensibility point; the engine machinery for inserting, indexing, querying, scoping, exporting, importing, and rolling back is identical across all kinds.

### 4.1 Principle of systematisation

Every fact-kind, no matter how exotic, follows the same seven stages:

| Stage | When | Form | What it does |
|---|---|---|---|
| **1 — DECLARE** | compile-time (grammar) | `@fact_kind: { … }` block | Defines the kind: attributes, requireds, indexes, scope behaviour, exportability. |
| **2 — EMIT** | parse-time (annotation on rule) | `@emit_fact { kind: K, … }` | Inserts an instance into the store on rule commit. |
| **3 — QUERY** | parse-time (annotation on rule) | `@predicate <name>(…)` | Reads from the store to gate a rule's commit or select a branch. |
| **4 — SCOPE** | parse-time (annotation on rule) | `@open_scope { kind: K, name: N }` / `@close_scope` | Pushes/pops a scope node in the parser's scope tree. |
| **5 — EXPORT** | parse-time (automatic, declared) | `exportable: true` in `@fact_kind` | On the enclosing scope's close, serialises matching facts to a library artefact. |
| **6 — IMPORT** | parse-time (annotation on rule) | `@import_from_library { kind: K, name: N }` | Lazily loads a library artefact and merges its facts into the current scope. |
| **7 — ROLLBACK** | parse-time (automatic, engine) | (no annotation; the `.3.3.3` IIFE boundary) | On `try_parse` abort, retracts every emit/scope-open performed during the transaction. |

**Every fact-kind walks this same protocol.** No kind opts out of any stage; no kind invents a parallel stage; no kind has special-case syntax. New kinds are pure additions to the schema; the engine's interaction with them is mechanical.

### 4.2 Stage 1 — DECLARE (compile-time)

Surface form, declared once per kind at the top of a grammar:

```ebnf
@fact_kind: {
  name:           <snake_case_kind_label>,
  attributes:     [ <attr_name>, <attr_name>, … ],
  required:       [ <subset_of_attributes> ],
  indexes:        [ ( <attr>, <attr>, … ), … ],
  scope_kind:     <opaque_scope_label>           # optional; default "current"
  exportable:     <bool>                          # optional; default false
  artefact_kind:  <opaque_artefact_label>         # optional; default kind name
  description:    "<one-line human description>"  # optional
}
```

Semantics:
- `name` — unique across the grammar; the kind label used everywhere in stages 2–7.
- `attributes` — ordered list of attribute names this kind carries. Engine treats them as opaque labels.
- `required` — subset of `attributes`; emit-time validation rejects facts missing any required attribute.
- `indexes` — list of composite-key index specifications. Engine maintains one index structure per spec. Default if omitted: `[(scope, kind, name)]`.
- `scope_kind` — which scope this kind lives in by default. If omitted, "current" (the innermost open scope at emit time).
- `exportable` — whether instances are eligible for library export. Default false.
- `artefact_kind` — directory name under `<lib-dir>/` for exported artefacts. Default = `name`.
- `description` — for documentation / `--explain` output.

Validation at codegen:
- `name` unique in this grammar.
- Every name in `required` and in each `indexes` tuple must appear in `attributes`.
- No duplicate attribute within a single index tuple.
- No zero-length index tuple.
- `scope_kind` must match a scope kind that the grammar uses in `@open_scope`.

Error on any violation = grammar-compile-time failure with a precise message; the grammar is rejected.

### 4.3 Stage 2 — EMIT (parse-time)

Surface form, attached to a grammar rule:

```ebnf
@emit_fact: { kind: <kind_name>, <attr_name>: <value_expr>, … }
<rule_name> := <body> -> <return_shape>
```

`<value_expr>` is a value-extraction expression in the existing semantic-annotation language: positional captures (`$N`), named captures (`$some_subrule`), dotted property access (`$x.body`, since `.3.3.4.a.1`), indexed access (`$x[0]`, since `.3.3.4.a.2`), or scalar literals.

Semantics — on rule commit (post body parse, pre return-shape construction):
1. Engine looks up `kind` in the registered `@fact_kind` declarations. If not found → grammar-compile-time error (already caught in Stage 1 validation).
2. Engine evaluates every supplied attribute's value expression against the rule's captures.
3. Engine validates all `required` attributes have a value. Missing → parse-time error with a precise message.
4. Engine determines the active scope per the kind's `scope_kind`.
5. Engine inserts the new fact into every index of this kind.
6. Engine records the insertion in the current transaction's undo log.

On rule reject (the `try_parse` discards this attempt): Stage 7 fires automatically; the fact never existed from the store's point of view.

### 4.4 Stage 3 — QUERY (parse-time)

Two surface forms, both attached to a rule:

**(a) Primitive query directly:**
```ebnf
@predicate has_fact args:[<kind_name>, <name_expr>] phase:<pre|branch|post>
@predicate fact_attribute_equals args:[<kind_name>, <name_expr>, <attr_name>, <value>] phase:<pre|branch|post>
@predicate fact_count_at_least args:[<kind_name>, <M>] phase:<pre|branch|post>
@predicate resolve_path args:[<dotted_name_expr>] phase:<pre|branch|post>
```

**(b) Composed named predicate:**
```ebnf
@predicate_def: {
  name: <named_predicate>,
  args: [ <arg_name>, … ],
  body: <expression-over-primitives>
}
```

then used as:
```ebnf
@predicate <named_predicate> args:[<arg_expr>, …] phase:<pre|branch|post>
```

Semantics:
- `phase: pre` — check before body parse; fail rule on false.
- `phase: branch` — used inside an alternation to gate which branch fires.
- `phase: post` — check after body parse, before commit; fail rule on false.
- The composed-predicate body is an expression over the primitives, parsed at grammar-compile time, optimised to a single store query when possible.

All four primitives are O(1) avg or O(log n) worst-case per §3.5; composed predicates retain that property by construction (they cannot loop unboundedly).

### 4.5 Stage 4 — SCOPE (parse-time)

Surface forms, attached to rules:

```ebnf
@open_scope: { kind: <scope_kind_label>, name: <name_expr> }
@close_scope
```

Semantics:
- `@open_scope` — engine pushes a new scope node onto the parser's active scope chain, with the given kind label (opaque) and name. The chain is a path in a scope **tree**: when the scope closes, its node remains in the tree (queryable for cross-scope lookups) but is removed from the active chain.
- `@close_scope` — engine pops the innermost scope. The popped node is preserved in the scope tree for archival queries (e.g., "list all class scopes from this file").

Scope kinds are opaque to the engine — the engine uses them only as labels for grouping and for `default_scope` lookups. Grammar discipline enforces meaningful labels (`package | class | function | block | covergroup | constraint_block | generate | interface | modport | …`); the set is unbounded.

### 4.6 Stage 5 — EXPORT (parse-time, automatic)

Triggered (not annotated explicitly on rules):

When a scope closes (Stage 4 `@close_scope`), the engine iterates over fact-kinds declared `exportable: true` whose facts live in that scope (by `scope_kind`), serialises them to `<lib-dir>/<scope-kind>/<scope-name>.<artefact_kind>.facts.json`, and atomically renames into place (same mechanism as `.3.3.4.a` MVP-0, extended to any kind).

The export is a side-effect of scope-close; grammar authors do not write export annotations per fact-kind. Whether a kind exports is declared once, in Stage 1.

### 4.7 Stage 6 — IMPORT (parse-time, explicit)

Surface form, attached to an import-rule:

```ebnf
@import_from_library: { kind: <artefact_kind>, name: <name_expr> }
```

Semantics:
1. Engine resolves `<lib-dir>/<scope_kind>/<name>.<artefact_kind>.facts.json` (the path used by Stage 5).
2. Engine deserialises lazily — only the index entries are loaded eagerly; attribute values are loaded on demand when first queried.
3. Imported facts are merged into the current scope as **imported facts** (a flag distinguishes them from locally-emitted facts; they participate in queries identically but are not eligible for re-export unless re-emitted locally).
4. If the artefact does not exist or fails the format-version check, engine raises a clean parse-time error.

### 4.8 Stage 7 — ROLLBACK (parse-time, automatic, engine-internal)

When a `try_parse` discards a speculative sub-parse, the engine's transaction wrapper (the `.3.3.3` IIFE pattern):
1. Reads the current transaction's undo log (one entry per fact emitted, one per scope opened, one per library imported within this transaction).
2. For each undo entry, removes the fact from every index it was inserted into, or pops the scope node, or unloads the lazy-import.
3. Discards the undo log.
4. Result: store is byte-identical to its state at transaction start.

Guarantees:
- Time: O(operations-undone), never O(store-size).
- Correctness: no fact emitted speculatively is ever observable after the transaction aborts.
- Composability: nested transactions (transaction inside transaction) compose; the inner transaction's undo log is folded into the outer's on inner-commit, or discarded on inner-abort.

### 4.9 Worked example — end-to-end for one fact-kind

To make the protocol concrete: here is the full lifecycle for one fact-kind — `variable_binding` — across all seven stages.

**Stage 1 — DECLARE (in `grammars/systemverilog.ebnf` near the top):**
```ebnf
@fact_kind: {
  name:           variable_binding,
  attributes:     [name, type_kind, type_ref, declared_in],
  required:       [name, type_kind],
  indexes:        [(scope, name), (scope, type_kind), (name)],
  scope_kind:     enclosing_block,
  exportable:     true,
  artefact_kind:  bindings,
  description:    "A bound identifier (var / parameter / port / class field / function-local) with its declared type."
}
```

**Stage 2 — EMIT (on the rule that binds a name):**
```ebnf
@emit_fact: { kind: variable_binding,
              name: $variable_name,
              type_kind: $resolved_type_kind,
              type_ref: $type_ref_body,
              declared_in: $current_scope_label }
variable_decl_assignment := … 
```

**Stage 3 — QUERY (at a method-call site that needs to know whether the receiver is an array):**
```ebnf
@predicate_def: { name: receiver_is_array,
                  args: [receiver_path],
                  body: resolve_path($receiver_path).attribute("type_kind") in ["array","queue","dynamic_array","assoc_array"] }

@branch_policy: predicate_first
method_call_body :=
        @predicate receiver_is_array args:[$enclosing_receiver] phase: branch
        built_in_method_call                                                  -> {kind: "built_in",        body: $1}
    |   …
```

**Stage 4 — SCOPE (at class / function / block boundaries):**
```ebnf
@open_scope: { kind: class, name: $class_name }
class_declaration := … @close_scope
```

**Stage 5 — EXPORT (automatic, because `exportable: true`):**
On `@close_scope` for class `Foo`, engine writes `<lib-dir>/class/Foo.bindings.facts.json` containing every `variable_binding` declared in the class's scope.

**Stage 6 — IMPORT (at a use-site that pulls in another package's bindings):**
```ebnf
@import_from_library: { kind: bindings, name: $package_name }
package_import_item := …
```

**Stage 7 — ROLLBACK (automatic):**
When parsing `class Foo { int a; int b; }` speculatively and the parser later backtracks, the two `variable_binding` facts emitted for `a` and `b` are removed from all three of their indexes — `(scope=Foo, name=a)`, `(scope=Foo, name=b)`, `(scope=Foo, type_kind=int)` (×2), `(name=a)`, `(name=b)` — and the scope node for class Foo is popped from the scope tree. Zero residue.

**The lifecycle is identical for every other fact-kind.** Adding a new kind (e.g., `covergroup_bin`, `macro_define`, `assertion_clock`, `constraint_membership`, anything we will think of later) means writing one Stage 1 declaration; the other six stages are mechanical. That is the systematisation the user is calling for, and the extensibility that lets us deal with any semantic type.

### 4.10 What the protocol guarantees (in one paragraph)

For any semantic fact in any future grammar: declaring its kind is one block of grammar code; emitting an instance is one annotation on the producer rule; querying an instance is one annotation (primitive or composed) at any consumer rule; scope, export, import, and rollback are either mechanically derived from the declaration or automatically handled by the engine. The protocol surface is small (seven stages), fixed (no new stages ever invented), and complete (covers every interaction the parser needs with the store).

## 5. Producers — facts to emit (decl-side semantic annotations)

For every grammar rule that BINDS a name, emit a typed fact. Concrete required additions (non-exhaustive — first cut, **`[TBC]`** during review):

| Decl rule | Fact to emit |
|---|---|
| `data_declaration` / `variable_decl_assignment` | `@emit_fact { kind: variable_binding, name: $name, type_kind: $resolved_type_kind, type_ref: $type_ref }` |
| `class_declaration` | `@emit_fact { kind: type_binding, name: $name, type_kind: "class" }` + `@open_scope` for members |
| `class_property` (= field) | `@emit_fact { kind: class_member, container: $enclosing_class, name: $name, member_kind: "field", type_kind: $type_kind, type_ref: $type_ref }` |
| `class_method` / `method_prototype` | `@emit_fact { kind: class_member, container: $enclosing_class, name: $name, member_kind: "method" }` |
| `parameter_port_declaration` / `tf_port_item` | `@emit_fact { kind: variable_binding, name: $name, type_kind: $type_kind, scope: $enclosing }` |
| `typedef` | `@emit_fact { kind: type_binding, name: $name, type_kind: $aliased_type_kind, body: $type_descriptor }` |
| `package_declaration` | `@open_scope kind:package, name:$name` (already done) |

The `type_kind` value is one of: `class | array | queue | dynamic_array | assoc_array | enum | scalar | interface | unresolved`.

**Open question (review):**
- **`[TBC]`** What exact shape do we want for `type_ref`? Bare string, or a structured object capturing element type + dimensionality? Tradeoff is fact-payload size vs. query precision.

## 6. Consumers — predicates to query (use-site semantic annotations)

At the disambiguation points in the grammar, the predicates that decide which alternative to take. Concretely for the method-call family:

```ebnf
@branch_policy: context_first      # NEW policy: take the predicate-true branch
method_call_body :=
        @predicate receiver_is_array args:[$enclosing_receiver]
        built_in_method_call                                                 -> {kind: "built_in", body: $1}
    |   @predicate receiver_is_class args:[$enclosing_receiver]
        method_identifier attribute_instance* lparen list_of_arguments rparen -> {kind: "method_call_with_args", ...}
    |   @predicate receiver_is_class args:[$enclosing_receiver]
        method_identifier attribute_instance*                                -> {kind: "method_bare_property", ...}
```

Where `$enclosing_receiver` is the receiver positional capture from the **parent** rule (`direct_method_call := method_call_root dot method_call_body` → method_call_body's parent gives us $1 = receiver).

**Predicates needed:**

- `receiver_is_array($r)` — true iff the resolved type of `$r` is `array | queue | dynamic_array | assoc_array`.
- `receiver_is_class($r)` — true iff the resolved type of `$r` is `class` (or class instance).
- `name_resolves($n)` — true iff `$n` resolves to a known binding in any active scope (for the cases where we want positive context but don't care about the specific kind).

The resolver underlying these predicates must do **multi-segment lookup**: given `seed_map.seed_table`, walk through `seed_map`'s class scope to find `seed_table`'s binding.

## 7. Engine primitives — gap analysis

Per §3.1, the engine should not encode a fixed taxonomy of fact-kinds — it must remain schema-agnostic and let grammar authors emit/query arbitrary kinds. The primitives below are the **minimal set** the engine must provide; everything else lives in grammar conventions.

| Primitive | Today | After this proposal |
|---|---|---|
| `@emit_fact { kind: K, name: N, <attr_K=V>* }` | ✓ | ✓ (no change; engine is schema-agnostic — new kinds emerge as conventions without engine modification) |
| Multi-index store (by `(scope,kind,name)`, by `kind`, by `(name)`, by attribute, by position) | partial (today: a single `(scope, kind, name)` map per `semantic_runtime_state`) | **extend** to maintain multiple indexes per fact-kind, declared via an optional per-kind index schema. Engine still doesn't know what the kinds mean; it just knows how to index them. |
| Transactional commit/rollback of emitted facts | ✓ (via `.3.3.3` IIFE pattern in `with_semantic_runtime_rule_transaction`) | ✓ — but the rollback must scale across all maintained indexes in O(emitted-during-tx), not O(store). The IIFE captures the tx boundary; the per-index undo lists live inside. |
| `@predicate has_fact(kind, name)` / `fact_attribute_equals(kind, name, attr, value)` / `fact_count_at_least(kind, M)` | ✓ | ✓ (kept verbatim; these are the composable query primitives §3.4 lists) |
| `@predicate resolve_path(dotted_name)` — multi-segment dotted lookup through nested scopes | ✗ | **NEW** — required for the `seed_map.seed_table.exists(...)` case and any future dotted-path disambiguation. Returns "fact-found-of-kind-K" or "unresolved". |
| Predicate references parent-rule positional capture (`$1` of parent) | ✗ | **NEW: `$enclosing_<role>`** or similar mechanism for child rules to reference parent siblings. **Alternative:** lift the predicate to the parent rule level so existing `$1` works — no engine change needed; see §8. |
| `@open_scope` / `@close_scope` (flat scopes) | ✓ (single global scope, plus `.3.3.4.a` library scope) | **extend** with **scope kind** (`file | package | class | function | block | …`); engine treats kinds as opaque labels (no hard-coded set); grammar authors enforce the discipline of consistent labelling. |
| Scope tree (parent/child relationships, walked outermost-first or innermost-first as the query demands) | partial (the IIFE preserves a "previous" state, which is a stack-style scope-tree) | **extend** to preserve the tree explicitly so cross-scope queries (e.g., "members of class C from outside C") work. |
| `@branch_policy: context_first` (select the predicate-true branch) | ✗ | **NEW** OR equivalent — branch-by-predicate selection. **Alternative:** factor each branch into a rule whose `@predicate has_fact phase: branch` gates it; existing `priority_first` policy may suffice if predicates fail cleanly. To be confirmed during §9 Phase 2. |
| `@export_to_library` / `@import_from_library` | ✓ (kind=package only, MVP-0) | **extend** to export *any* kind (not just `type_name`) — same schema-agnostic stance as the in-memory store; grammars declare which kinds are exportable per library. |
| Per-fact-kind index schema declaration (e.g., `@index_schema: { kind: class_member, indexes: [(scope,name), (container), (kind)] }`) | ✗ | **NEW** — optional; defaults to `(scope, kind, name)`. Grammar-level declaration; engine consumes it to build indexes at codegen-or-load time. |

### Summary — the real engine extensions

1. **Multi-index store with schema-agnostic emission** — engine maintains as many indexes per fact-kind as the grammar declares; the engine itself never enumerates known kinds. This is the §3 generalisation made operational.
2. **Multi-segment dotted-path resolver** (`resolve_path`).
3. **Scope-kind labels + explicit scope tree** (so cross-scope queries work).
4. **Branch-by-predicate** at the rule level — *probably* expressible with existing `phase: branch` predicates if we lift the disambiguation up one rule. **`[TBC]` — verify before adding a new primitive.**
5. **Transactional rollback across all indexes** (O(emitted-in-tx), reusing the `.3.3.3` IIFE boundary).

Items 1–3 are independent of method-call disambiguation and benefit every future grammar (regex, VHDL, RTL, anything). They are the parser-agnostic foundation; the SV-specific consumers in §6 build on top of them.

## 8. Open questions for review

- **§5 Producers** — is the proposed fact-kind set complete enough for uvm? What about for ports / interface modports / `localparam` / `generate`-emitted names?
- **§6 Consumers** — should the predicates be named (`receiver_is_array`) or expressed via composable boolean operators on facts (`has_fact(variable_binding, $r) AND fact_attribute(variable_binding, $r, type_kind, array)`)? Tradeoff: named predicates are easier to read, composable ones are more general.
- **§7 Engine primitives** — do we add `$enclosing_receiver` (capture-from-parent), or lift the predicate to the parent rule level so existing `$1` works? The latter is grammar-only (no engine change) but requires restructuring; the former is engine-level (one new primitive) but keeps the grammar shape.
- **Failure mode when context is unknown** — per §1 there is no fallback. So what does the parser do when it hits `obj.method(args)` and `obj` is genuinely unresolved (e.g., parsed in isolation, no library)? Options:
  - **(a)** the parse is a hard error — "unresolved receiver, cannot disambiguate".
  - **(b)** a deliberate "unresolved" type kind that has its own queryable predicate, and the grammar has an explicit alternative for the unresolved case (still context-aware, just one of the contexts is "I don't know").
  - **`[TBC]`** — preference?

## 9. Phased implementation plan (proposal — for amendment)

- **Phase 0 (this doc)** — design proposal review + amendment until landed.
- **Phase 0.5 — Sign-off artefacts (§11.1).** Before any code: API contract document + schema-language spec (the §3.6 `@fact_kind:` declaration block + the §4 lifecycle protocol surface) + test plan + performance contract. Reviewed and approved. **Leaf:** `.3.3.4.b.5.0`.
- **Phase 1 — Engine: universal semantic store + lifecycle protocol implementation (§3 + §4).** Multi-index schema-agnostic fact store, per-kind index-schema declaration, scope-kind labels + explicit scope tree, multi-segment dotted-path resolver, transactional rollback across all indexes, and the seven-stage lifecycle (DECLARE / EMIT / QUERY / SCOPE / EXPORT / IMPORT / ROLLBACK) wired end-to-end. Parser-agnostic. Verified by a tiny synthetic grammar exercising multiple fact-kinds with different index schemas + nested scopes + dotted lookup + speculative-parse rollback + library export/import round-trip — every stage of the protocol exercised at least once. **Leaf:** `.3.3.4.b.5.1`.
- **Phase 2 — Engine: branch-by-predicate (or grammar-restructure equivalent).** Decide between the two paths during Phase 1 (verify whether existing `phase: branch` predicates can reference parent-rule captures). **Leaf:** `.3.3.4.b.5.2`.
- **Phase 3 — Engine: extend `@export_to_library` / `@import_from_library` to richer fact kinds.** **Leaf:** `.3.3.4.b.5.3`.
- **Phase 4 — SV grammar producer pass.** Add `@emit_fact` to every decl-site rule per §5. No other changes. Verify facts emit correctly via a probe. Re-run uvm corpus — parse still fails (consumers not yet wired), but no regression elsewhere. **Leaf:** `.3.3.4.b.6.1`.
- **Phase 5 — SV grammar consumer pass.** Wire predicates from §6 into the method-call disambiguation rules. Drop the symptomatic `( lparen list_of_arguments rparen )?` workarounds. Re-run uvm — expect parse to succeed on the diagnosed construct. **Leaf:** `.3.3.4.b.6.2`.
- **Phase 6 — verification + lockstep.** Full triage gate; lib tests; RGX 44/0; SV shape-contract; book lockstep; release bump 1.0.126 → 1.0.127. **Leaf:** `.3.3.4.b.6.3`.

Each leaf commits independently with its own contract bump and lockstep. No leaf depends on a future leaf to be valid.

## 10. Non-goals (deliberately out of scope)

- Full SV type checking. We need *enough* type information to disambiguate the grammar — not enough to type-check expressions.
- LRM-strict adherence in cases where LRM permissiveness is the bug we're fixing. The LRM's BNF is the *spec for what is syntactically valid*; our PGEN grammar is the *spec for what we parse, which is a deterministic subset chosen via context*.
- Parser-specific features. Every engine extension here must be applicable to VHDL, RTL, and future grammars — `feedback_ast_pipeline_parser_agnostic`.

## 11. Quality bar — sign-off level requirements

User direction, 2026-05-21: "The semantic-store shall be top-notch, sign-off level quality. It shall be designed and built with a lot of care, because it is going to stay with us for a long, long time, it better be very good."

This subsystem is **not** scoped as a tactical implementation. It is a foundational engine module that every future grammar — VHDL, RTL, regex extensions, languages we haven't yet considered — will depend on. The quality bar reflects that.

### 10.1 Design before code

Phase 1 (`.3.3.4.b.5.1`) does **not** start with implementation. It starts with:

- A formal **API contract** document (`docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md`) describing every primitive: emit, query, scope open/close, transaction begin/commit/abort, import/export, schema declare. With pre/postconditions, error modes, complexity guarantees, and stability guarantees (which parts are public-stable, which are internal).
- A **schema definition language spec** (extends `grammars/semantic_annotation.ebnf` with the `@fact_kind:` declaration block of §3.6).
- A **test plan** enumerating: unit tests per primitive, property tests for invariants (insert-then-lookup, rollback-leaves-no-trace, scope-walk-finds-nothing-after-close), stress tests at scale (≥1M facts, ≥100k scopes), adversarial tests (malformed schema, conflicting kinds, query on nonexistent index).
- A **performance contract**: target numbers for each primitive (e.g., `has_fact` ≤ 200ns p99 at 1M facts; library import lazy with ≤ 10ms cold-start; rollback ≤ 1µs per emitted fact). Continuous benchmarks landed alongside the implementation.

Code lands only after these four artefacts are reviewed.

### 10.2 API stability

The public API of the semantic store must be **versioned**. Once a primitive is published, breaking changes require:

- A deprecation marker + parallel migration path.
- Bumping a major version of `PGEN_SEMANTIC_STORE_API_CONTRACT.md`.
- A documented migration cookbook in the contract.

Internal layout (which indexes are maintained, how arenas are sized, etc.) may change freely without API impact.

### 10.3 Observability + diagnostics

The store must be **debuggable in production** without recompilation:

- Per-primitive operation counters (number of `@emit_fact`, number of `has_fact`, number of rollbacks, …).
- Per-index hit/miss/scan-size counters.
- Per-fact-kind population counters.
- An `--explain` mode for any predicate query: which index was used, how many rows scanned, total wall-clock time.
- A library-artefact dump tool that prints a human-readable view of any persisted facts (including: schema, indexes, sample queries).

Hooked into `trace_log` so it integrates with the existing `PGEN_TRACE_VERBOSITY` mechanism.

### 10.4 Library artefact format stability

Library artefacts (`<lib-dir>/<kind>/<name>.facts.json` from `.3.3.4.a`) are **persisted state**. Their format must be:

- Versioned (`format_version` field, already present in `.3.3.4.a` at `1`).
- Forward-readable: a newer parser must be able to read an older artefact (additive only).
- Backward-readable where feasible: an older parser can read a newer artefact's compatible subset, or error cleanly with a clear message.
- Documented in `PGEN_SEMANTIC_STORE_API_CONTRACT.md` §library-artefact-format.
- Migrated by a documented procedure when the format changes (with a `pgen migrate-library` tool).

### 10.5 Testing standard

Every primitive landed in this subsystem must ship with:

- Unit tests in the `tests/` directory for the primitive in isolation.
- Property tests (using `proptest` or equivalent) for invariants — particularly the rollback-no-trace, lookup-after-emit, and scope-walk-correctness properties.
- A perf bench (`cargo bench`) anchored to the §11.1 performance contract numbers.
- An end-to-end test through a synthetic grammar that uses the primitive via `@emit_fact` / `@predicate` and verifies the observable behaviour.

CI gates regress on any of these; the next slice can land only when all gates are GREEN.

### 10.6 What "sign-off" means concretely

By analogy with IC tape-out: a subsystem at sign-off has been **independently validated** against its contract, has **no known correctness defects**, has **measured performance within budget**, and has **documented behaviour for every API surface**. For PGEN this translates to:

- The API contract has been reviewed (Phase 1 design review).
- 100% of public API surface has unit tests + property tests.
- 100% of public API surface has documented complexity and error semantics.
- Performance bench numbers are within budget on a baseline machine (recorded in the contract).
- The schema-declaration language has its own test corpus and is exercised end-to-end by at least one grammar (synthetic in Phase 1; SV in Phase 4).
- Library artefacts have a documented format spec with at least one migration scripted (proves the migration mechanism works).

Nothing less ships under this banner.

## 12. Restore points

- Pre-design baseline (current HEAD): `checkpoint/post-3-3-4-b-3-layer-0-clean-pre-next-lrm-defect` @ `4195ee22`.
- Pre-Layer-0 baseline (kept for reference): `checkpoint/post-3-3-4-b-1-clean-pre-layer-0` @ `f758b878`.

## 13. References

- `feedback_ast_pipeline_parser_agnostic` (memory) — every pipeline change must be a general primitive.
- `feedback_prefer_grammar_leave_engine_alone` (memory, refined) — engine changes ARE on the table when they're parser-AGNOSTIC features that make the EBNF cleanly express what the language needs.
- `feedback_verify_rule_correctness_before_runtime_hypotheses` (memory) — when a parse failure is tied to a rule, READ THE RULE AGAINST THE SPEC FIRST. (This proposal extends the discipline: also, ASK WHETHER THE LRM HAS GIVEN US ENOUGH TO DISAMBIGUATE — and if not, that's the disambiguation we owe via context.)
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` §1.0.121 — the `.3.3.3` IIFE exception-safety fix that landed `@open_scope`/`@close_scope`/`@emit_fact`/`has_fact` reliably. This proposal builds on that.
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` §1.0.122 — the `.3.3.4.a` MVP-0 library mechanism. This proposal extends its fact-kind set.

---

**Amend freely.** When the principle in §1 and the open questions in §8 are settled, we can start scoping Phase 1.
