# The Semantic Store: Parser Memory

This chapter introduces the **semantic store** — PGEN's parser-memory subsystem.
You can think of it as a small, fast database that the parser populates as it
goes, and queries to decide what to parse next. The store is **parser-agnostic**
(every grammar uses the same primitives), **schema-agnostic** (new fact-kinds
are declared in the grammar, not the engine), and **transactional** (speculative
parses leave no trace if they fail).

If you're new to PGEN's semantic annotations, skim
[Annotation System](annotation-system.md) first for the broad strokes (return
annotations vs semantic annotations, the `@directive: {...}` surface, etc.).
This chapter zooms in on **how the parser remembers what it has seen and uses
that to steer parsing decisions**.

---

## 1. Why does the parser need memory?

Start with a SystemVerilog construct that looks deceptively simple:

```sv
if (seed_map.seed_table.exists(type_id)) begin
  // ...
end
```

What kind of expression is `seed_map.seed_table.exists(type_id)`? It depends
on what `seed_map` *is*:

- If `seed_map` is a **class instance** whose member `seed_table` is an
  **associative array**, then `.exists(type_id)` is a built-in array-method
  call.
- If `seed_map` is a class instance whose member `seed_table` is a **function
  returning an object**, then `.exists(type_id)` is a regular method call on
  the returned object.
- If `seed_map.seed_table` is a **hierarchical name path**, then `exists`
  could be a free function in some scope and `(type_id)` is a positional
  call.

All three interpretations are syntactically valid SystemVerilog. The LRM's
grammar can express each of them, but it cannot tell the parser *which*
applies — that requires knowing what `seed_map` was declared as.

Without memory, a PEG parser would have to guess (typically "first
alternative that syntactically fits wins"). That gives the so-called
"drunk-stumbling" parser: it commits to whichever path syntactically
succeeds first, regardless of whether the parse it produces is actually
correct in context.

**With memory, the parser knows.** When `seed_map` was declared, a fact was
emitted saying "seed_map is a class instance of type X". When the parser
reaches the method-call site, it queries that fact and routes
deterministically to the right rule.

The same pattern applies to every context-dependent grammar decision: type
disambiguation, scope-qualified name resolution, macro expansion, generate
unrolling, cross-file imports. The semantic store is the single primitive
that supports them all.

> **The principle:** the parser is a process with memory. Semantic
> annotations are how that memory is built, organised, scoped, and queried.
> With the right context at the right time, the parser always knows which
> rule to take.

## 2. The mental model

The semantic store has three moving parts you'll meet again and again:

1. **Facts.** A fact is `(kind, name, attributes)`: a category label, a
   name, and an unordered key-value bag. Every emit produces one fact; every
   query asks about facts.
2. **Scopes.** Facts live in scopes. Scopes form a tree (package contains
   classes; classes contain methods; methods contain blocks). When you query
   for a fact, the store walks the active scope chain from innermost to
   outermost — exactly the visibility rule you'd want.
3. **Transactions.** Every parse attempt happens inside a transaction. If
   the attempt fails (PEG backtracking), every fact emitted, every scope
   opened, and every library imported during that transaction is **rolled
   back atomically**. The store ends up byte-identical to its pre-transaction
   state. This is what lets PEG's speculation work cleanly: speculative
   parses don't pollute the global view.

That's the whole model. Three concepts, fixed semantics, no exceptions.

## 3. The lifecycle protocol

The interaction between a grammar and the semantic store follows a single
prescribed lifecycle of **seven stages**. Every fact-kind walks all seven
stages — no opt-outs, no inventing parallel patterns. This is the
"systematisation" guarantee: there's exactly one way to do each operation.

| Stage | When | Form | What it does |
|---|---|---|---|
| **1 — DECLARE** | grammar compile-time | `@fact_kind: {...}` | Defines a new fact-kind (its attributes, requireds, indexes, scope, exportability). |
| **2 — EMIT** | rule commit (parse-time) | `@emit_fact: { kind: K, ... }` | Records one fact in the store. |
| **3 — QUERY** | rule pre / branch / post (parse-time) | `@predicate <name>(...)` | Reads from the store to gate or steer parsing. |
| **4 — SCOPE** | rule entry / exit (parse-time) | `@open_scope` / `@close_scope` | Pushes / pops a scope node on the tree. |
| **5 — EXPORT** | scope close (automatic) | declared via `exportable: true` | Writes facts to a library artefact for cross-file reuse. |
| **6 — IMPORT** | rule body (parse-time) | `@import_from_library: {...}` | Lazily loads facts from a library artefact into the current scope. |
| **7 — ROLLBACK** | speculative-parse abort (automatic) | (engine-internal) | Undoes every emit / scope-open / import done in the aborted transaction. |

Sections 4–10 walk through each stage with concrete examples. Pick the
chapter you need — they're independent reads.

The rest of this chapter is the user-facing surface. For deeper material
(the design rationale, the universal-store architecture, the multi-index
performance contract) see [`docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md`](../../proposals/CONTEXT_AWARE_PARSING_DESIGN.md)
and the four sign-off contracts under [`docs/contracts/`](../../contracts/).

## 4. Stage 1 — DECLARE: `@fact_kind:`

To use a new kind of fact, declare it once at the top of your grammar:

```ebnf
@fact_kind: {
  name:           variable_binding,
  attributes:     [name, type_kind, type_ref, declared_in],
  required:       [name, type_kind],
  indexes:        [(scope, name), (scope, type_kind)],
  scope_kind:     enclosing_block,
  exportable:     true,
  artefact_kind:  bindings,
  description:    "A bound identifier with its declared type."
}
```

That's all. The engine now knows:

- This grammar can emit facts of kind `variable_binding`.
- Each instance carries the four named attributes.
- `name` and `type_kind` are mandatory (emit-time validation catches
  omissions).
- The engine maintains two secondary indexes for fast lookups.
- The facts live in the innermost enclosing block scope (functions,
  for-blocks, generate-blocks, etc.).
- When such a scope closes, the engine writes the facts to a library
  artefact under `<lib-dir>/<scope-name>.bindings.facts.json`.

### Field reference

| Field | Required | Default | Purpose |
|---|---|---|---|
| `name` | **yes** | — | Identifier label for this kind. Must be unique in the grammar. snake_case convention. |
| `attributes` | **yes** | — | The keys that fact instances carry. Non-empty list. |
| `required` | no | `[]` | Subset of `attributes` that must be present at emit time. |
| `indexes` | no | `[(scope, kind, name)]` | Secondary indexes (composite keys) the engine maintains for fast lookup. |
| `scope_kind` | no | `current` | Which scope kind these facts default to. |
| `exportable` | no | `false` | Whether instances are written to library artefacts on scope close. |
| `artefact_kind` | no | `name` | Subdirectory under `<lib-dir>` for exported artefacts. |
| `description` | no | empty | Human-readable description (shown in `--explain` output). |

### Validation cookbook

The engine enforces seven validation rules (V-DECL-1 through V-DECL-7) at
grammar-compile time. Concrete examples of what passes and what fails:

```ebnf
# ✓ Valid: minimal well-formed declaration.
@fact_kind: { name: foo, attributes: [name] }

# ✗ V-DECL-2 — attributes must be non-empty.
@fact_kind: { name: foo, attributes: [] }

# ✗ V-DECL-3 — `required` references an attribute not in `attributes`.
@fact_kind: { name: foo, attributes: [name], required: [type_kind] }

# ✓ V-DECL-4 carve-out — `scope` and `kind` are always indexable.
@fact_kind: { name: foo, attributes: [name], indexes: [[scope, kind, name]] }

# ✗ V-DECL-4 — an index tuple references an undeclared attribute.
@fact_kind: { name: foo, attributes: [name], indexes: [[scope, missing]] }

# ✗ V-DECL-5 — index tuples cannot be empty.
@fact_kind: { name: foo, attributes: [name], indexes: [[]] }

# ✗ V-DECL-5 — no duplicates within a tuple.
@fact_kind: { name: foo, attributes: [name], indexes: [[name, name]] }

# ✗ V-DECL-7 — kind names and artefact_kinds must be valid path components.
@fact_kind: { name: "../escape", attributes: [x] }
@fact_kind: { name: ok, attributes: [x], artefact_kind: ".hidden" }
```

V-DECL-1 (uniqueness across the grammar) is checked when the grammar is
compiled. Identical re-declarations are explicitly allowed — you can attach
the same `@fact_kind:` block to multiple rules without conflict; conflicting
payloads with the same name are rejected.

V-DECL-6 (the `scope_kind` field references a known scope label) is a
**warning** at compile time, not an error — grammars evolve and may
temporarily reference scope kinds that aren't yet wired up.

## 5. Stage 2 — EMIT: `@emit_fact:`

Attach `@emit_fact:` to the rule where the binding happens:

```ebnf
@emit_fact: { kind: variable_binding,
              name: $variable_name,
              type_kind: $resolved_type_kind,
              type_ref: $type_descriptor }
variable_decl_assignment := type_descriptor declared_identifier ...
```

When that rule successfully parses, the engine:

1. Validates `kind` references a declared `@fact_kind`.
2. Resolves attribute expressions (`$variable_name`, `$resolved_type_kind`,
   `$type_descriptor`) against the rule's captures.
3. Checks every `required` attribute has a value.
4. Inserts the new fact into the master Vec **and** every secondary index.
5. Records the insertion in the active transaction's undo log.

If the rule fails to commit (PEG backtracks), the fact is automatically
removed from every index — Stage 7 handles this without you having to think
about it.

### Attribute value expressions

The expressions you put in `@emit_fact:` attribute slots are the same
expressions you use in return annotations: positional captures (`$1`),
named captures (`$variable_name`), dotted property access (`$x.body`),
indexed access (`$items[0]`), and scalar literals. See
[`docs/RETURN_ANNOTATIONS_REFERENCE.md`](../../RETURN_ANNOTATIONS_REFERENCE.md)
for the full surface.

## 6. Stage 3 — QUERY: `@predicate`

There are two ways to query: directly invoke a built-in primitive, or
compose primitives into a named predicate.

### Built-in primitives

The engine provides four:

```ebnf
# Does any fact of this (kind, name) exist in scope?
@predicate has_fact args:[variable_binding, $1] phase: post

# Does a fact have a specific attribute equal to a specific value?
@predicate fact_attribute_equals args:[variable_binding, $1, type_kind, array] phase: branch

# Are there at least M facts of this kind?
@predicate fact_count_at_least args:[capture_group, 1] phase: pre

# Resolve a dotted path through scopes (coming in .b.5.1.4).
@predicate resolve_path args:[$dotted_name] phase: branch
```

Each primitive completes in **average O(1)** thanks to the per-kind secondary
indexes. (See §9 for the performance story.)

### Phases

The `phase:` modifier tells the engine *when* to check:

- `pre` — before the rule's body parses. Failure rejects the rule
  immediately (saves work).
- `branch` — inside an ordered choice, gating which branch fires.
- `post` — after the rule's body parses, before committing the
  transaction. Failure backs out the rule plus its fact emissions.

### Composed predicates

For complex conditions you can define a named predicate that composes the
primitives:

```ebnf
@predicate_def: {
  name: receiver_is_array,
  args: [receiver_path],
  body: resolve_path($receiver_path).attribute("type_kind") in ["array", "queue", "dynamic_array", "assoc_array"]
}
```

Then use it like a primitive — in any phase, including `phase: branch`:

```ebnf
@predicate receiver_is_array args:[$receiver] phase: branch
```

A composed predicate is a drop-in replacement for a built-in predicate
everywhere a predicate is accepted: there is no separate "branch-by-predicate"
construct. When attached with `phase: branch` it gates the choice exactly as a
built-in does — the branch fires when the composed body evaluates to true, is
skipped when it evaluates to false, and is left unblocked when the body is
*indeterminate* (for example, a `resolve_path` that finds nothing). If you need
an unknown receiver to actively block a branch rather than fall through, author
the body so the unknown case returns false instead of indeterminate.

The body language is small on purpose: boolean operators (`&&`, `||`, `!`),
comparisons (`==`, `!=`, `<`, ...), set membership (`in [...]`), attribute
access (`.attribute("name")`). No recursion, no arithmetic — predicates are
*decisions*, not computations.

## 7. Stage 4 — SCOPE: `@open_scope` / `@close_scope`

Facts live in scopes. Scopes form a tree. You declare scope boundaries with:

```ebnf
@open_scope: { kind: class, name: $class_name }
class_declaration := kw_class declared_identifier ... kw_endclass
                  -> {kind: "class", name: $2, body: $4}
                  @close_scope
```

When the rule starts parsing, `@open_scope` pushes a new node onto the
active scope chain. When the rule commits, `@close_scope` pops it (the node
stays in the tree, available for archived queries via future
`resolve_path`-style lookups).

Common scope kinds you'll encounter: `global`, `file`, `package`, `class`,
`interface`, `function`, `task`, `block`, plus custom labels grammar
authors define for domain-specific scopes (generate blocks, covergroups,
constraint blocks, etc.).

## 8. Stage 5 + 6 — EXPORT / IMPORT: cross-file facts

When a grammar parses a single file but its constructs reference names
declared in *other* files (`import pkg::*`, includes, etc.), the engine
needs a way to share facts between parse sessions. That's the **library
mechanism**, introduced in `.3.3.4.a` and extended by Stage 5/6 of the
protocol.

### Stage 5 — Export

Export has two halves, and they are deliberately separate:

1. **Where** facts are written — an `@export_to_library` directive on a
   scope-defining rule (a `package_declaration`, a `class_declaration`, …).
   When that rule commits, the engine takes the facts the rule's subtree
   emitted and writes them to a library artefact.
2. **Which** facts are written — the `exportable` flag on each
   `@fact_kind:` declaration. Only facts whose kind is declared
   `exportable: true` are persisted; everything else stays parse-local.

```ebnf
@fact_kind: {
  name: type_binding,
  attributes: [name, kind],
  exportable: true,
  artefact_kind: types
}
```

So you declare a kind exportable once, in its `@fact_kind:` block, and
every `@export_to_library` point automatically persists facts of that kind
— and *only* of kinds marked exportable. A kind you emit for in-parse
queries but never want on disk simply leaves `exportable` at its default
(`false`).

This is fully schema-agnostic: **any** kind a grammar declares exportable
round-trips through the library, not a fixed built-in set. (A grammar that
has not yet declared any `@fact_kind:` schema at all keeps a conservative
transitional default so it does not lose export behaviour before it
migrates.) The artefact is written atomically — a temp file then a rename
— so a crash mid-write never leaves a half-written library.

### Stage 6 — Lazy import

To pull a library artefact's facts into the current parse, attach an
explicit import directive:

```ebnf
@import_from_library: { kind: types, name: $package_name }
package_import_item := kw_import package_identifier "::" ...
```

The engine resolves `<lib-dir>/<scope-kind>/<package_name>.types.facts.json`
and merges its facts into the current scope. The load is **lazy** — only
the index entries are loaded eagerly; attribute values are loaded on demand
when first queried. So importing a giant `uvm_pkg` library doesn't stall
the parser.

## 9. Stage 7 — ROLLBACK: speculation-safe by construction

PEG parsers backtrack. They try one alternative, parse partway through,
discover it doesn't fit, and roll back to try the next. Without
transactional rollback, every fact emitted during the failed alternative
would pollute the store. Queries would see ghost facts. Decisions would be
non-deterministic.

The engine wraps every speculative parse attempt in a transaction. When the
attempt fails, the engine:

1. Reads the transaction's undo log.
2. Removes each undo-logged fact from every index it was inserted into.
3. Pops every scope-open performed during the transaction.
4. Discards every speculative library import.

After rollback, the store is **byte-identical** to its state before the
transaction started.

You don't write any annotation for Stage 7. It's automatic, courtesy of the
generator's `with_semantic_runtime_rule_transaction` wrapper (the
`.3.3.3` IIFE-pattern fix that guarantees the rollback fires on every
non-commit exit path).

The rollback cost is **O(operations undone)** — never proportional to total
store size — so even heavy PEG backtracking (the SystemVerilog grammar
exhibits hundreds of restorations per parse) stays well within the
performance budget.

## 10. The multi-index performance story

Behind the scenes, the store maintains multiple indexes per fact-kind. Each
index is a hash map keyed on a composite tuple (declared in `@fact_kind`'s
`indexes:` field). When you call `has_fact(K, N)`, the engine consults the
`(scope, kind, name)` index — one hash lookup, O(1) average.

Before sub-leaf `.3.3.4.b.5.1.1`, every predicate walked the full fact
vector linearly:

```rust
// Old: O(N) per query
self.facts.iter().any(|fact| fact.kind == k && fact.name == n)
```

At uvm scale (~50k facts), that's ~50k comparisons per query, and
predicates fire on every grammar rule. The store would become the parser's
bottleneck.

After multi-index:

```rust
// New: O(1) average per query
self.fact_index.any_with_name(k, n)
```

The same predicate now consults a hash bucket directly — independent of
store size. The Rust types involved are private to `semantic_runtime.rs`,
but you can see the perf budgets in the
[performance contract](../../contracts/PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md):
`has_fact` ≤ 200ns p99, `fact_count_at_least` ≤ 50ns p99, and so on.

The take-away for grammar authors: **you don't change anything**.
Predicates and `@fact_kind:` declarations look identical regardless of
whether the engine internally uses linear scans or hash indexes. The
performance comes for free.

## 11. A worked example end-to-end

Here is the entire lifecycle for one fact-kind — `variable_binding` —
walked through all seven stages.

```ebnf
# -------- Stage 1: declare the kind --------
@fact_kind: {
  name:           variable_binding,
  attributes:     [name, type_kind, type_ref],
  required:       [name, type_kind],
  indexes:        [(scope, name), (scope, type_kind), (name)],
  scope_kind:     enclosing_block,
  exportable:     true,
  artefact_kind:  bindings,
  description:    "A bound identifier with its declared type."
}

# -------- Stage 4: define scope boundaries --------
@open_scope: { kind: class, name: $1 }
class_declaration := kw_class declared_identifier
                     class_body
                     kw_endclass
                  -> {kind: "class", name: $2, body: $3}
                  @close_scope

# -------- Stage 2: emit on each variable decl --------
@emit_fact: { kind: variable_binding,
              name: $variable_name,
              type_kind: $resolved_type_kind,
              type_ref: $type_ref_body }
variable_decl_assignment := type_descriptor declared_identifier ...

# -------- Stage 3: query at use sites --------
@predicate has_fact args:[variable_binding, $1] phase: post
known_unscoped_variable_identifier := simple_identifier

# -------- Stage 6: import from another package --------
@import_from_library: { kind: bindings, name: $package_name }
package_import_item := kw_import package_identifier "::" ...
```

What happens at parse time:

1. The parser enters a class declaration. **Stage 4** opens a `class` scope
   named after the class identifier.
2. Inside, each `variable_decl_assignment` emits one `variable_binding`
   fact (**Stage 2**). The fact lands in the master Vec and three indexes.
3. Later, a `known_unscoped_variable_identifier` use-site queries
   (**Stage 3**): does a `variable_binding` named `$1` exist? If yes, the
   rule commits; if no, the rule fails and the parser backtracks.
4. If the rule succeeds, the class scope closes (**Stage 4**). At that
   moment, **Stage 5** fires automatically because `exportable: true` —
   every `variable_binding` in this class's scope is written to
   `<lib-dir>/class/<class-name>.bindings.facts.json`.
5. Elsewhere, a `package_import_item` rule that resolves a different
   package triggers **Stage 6** — the engine lazily loads that package's
   bindings into the current scope.
6. If at any point the parser hits a backtrack, **Stage 7** unwinds every
   emit / scope-open / import that happened in the failed transaction.

**Adding a new fact-kind** — say `class_member`, or `covergroup_bin`, or
`assertion_clock` — is one new `@fact_kind:` block plus annotations on the
relevant rules. The seven-stage protocol applies identically. No engine
change needed.

## 12. Where to learn more

- [`docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md`](../../proposals/CONTEXT_AWARE_PARSING_DESIGN.md) —
  the full design, including the universal-store rationale and the
  architectural principles.
- [`docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md`](../../contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md) —
  the public engine API, with preconditions, postconditions, error modes,
  and stability classes.
- [`docs/contracts/PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md`](../../contracts/PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md) —
  the formal EBNF for every `@fact_kind` / `@emit_fact` / `@predicate` /
  `@open_scope` / `@close_scope` / `@import_from_library` form.
- [`docs/contracts/PGEN_SEMANTIC_STORE_TEST_PLAN.md`](../../contracts/PGEN_SEMANTIC_STORE_TEST_PLAN.md) —
  the cases that must pass before each sub-leaf lands.
- [`docs/contracts/PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md`](../../contracts/PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md) —
  the latency, memory, and scalability budgets.
- [Annotation System](annotation-system.md) — the broader annotation
  surface (return annotations + semantic annotations, the two-family model).

## 13. Cheat sheet

```ebnf
# Declare a fact-kind (once, at the top of the grammar):
@fact_kind: { name: <ident>, attributes: [<names>],
              required: [<subset>]?, indexes: [(<tuple>), ...]?,
              scope_kind: <ident>?, exportable: <bool>?,
              artefact_kind: <ident>?, description: <string>? }

# Emit a fact (on rule commit):
@emit_fact: { kind: <kind>, <attr>: <expr>, ... }
<rule_name> := <body> -> <return_shape>

# Query (any phase: pre / branch / post):
@predicate <name> args:[<args>] phase: <phase>
<rule_name> := <body>

# Open / close a scope (around a region):
@open_scope: { kind: <ident>, name: <expr> }
<rule_name> := <body> @close_scope

# Import a library (per rule):
@import_from_library: { kind: <artefact_kind>, name: <expr> }
<rule_name> := <body>

# Export — no annotation needed; declare `exportable: true` in @fact_kind.
# Rollback — no annotation needed; automatic on transaction abort.
```

That's the entire user-facing surface of the semantic store. Seven stages,
prescribed forms, no improvisation, and engine machinery designed to scale.
