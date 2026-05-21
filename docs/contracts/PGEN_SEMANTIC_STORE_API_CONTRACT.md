# PGEN Semantic Store — API Contract

**Status:** DRAFT (sketch — first cut for review).
**Spec version:** 0.1.0 (pre-1.0 — breaking changes allowed during sketch).
**Stability commitment:** none yet — none of this is approved.
**Reviewers required before 1.0.0:** project owner (PGEN). After 1.0.0 every breaking change requires a deprecation marker + migration cookbook + major-version bump.
**Companion documents (Phase 0.5 deliverables):**
  - `docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md` (the design)
  - `grammars/semantic_annotation.ebnf` extensions (the schema surface; separate artefact)
  - `docs/contracts/PGEN_SEMANTIC_STORE_TEST_PLAN.md` (separate artefact)
  - `docs/contracts/PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md` (separate artefact)

---

## 1. Purpose and scope

This contract specifies the public API of the PGEN **Semantic Store** subsystem — the universal, schema-agnostic, multi-indexed, scope-aware, transactional fact database the parser uses to navigate context-dependent grammar decisions (per `CONTEXT_AWARE_PARSING_DESIGN.md` §3 / §4).

Two surfaces are specified:

- **Engine API** (Rust-level, §3) — what `rust/src/ast_pipeline/semantic_runtime.rs` (or its successor) exposes to the rest of the generator + generated parsers.
- **Annotation surface** (grammar-author-level, §4) — what `@fact_kind:` / `@emit_fact` / `@predicate` / `@open_scope` / `@import_from_library` look like in a `*.ebnf` grammar.

Both surfaces are versioned together (§2). Internal layout (which indexes are kept, arena sizing, hash function choice, etc.) is **not** part of the contract and may change between minor versions.

## 2. Versioning and stability classes

This contract follows SemVer. Public API elements are tagged with one of three stability classes:

- **`Stable`** — covered by SemVer. Breaking changes require a major-version bump + deprecation cookbook.
- **`Provisional`** — public but explicitly subject to change before 1.0.0 (or before being promoted to Stable in a minor release after 1.0.0). Use with awareness.
- **`Internal`** — exposed for in-tree use only; no compatibility guarantee. Out-of-tree consumers must not depend on these.

Until this document is reviewed and accepted, **every element is Provisional**. The sketch deliberately does not lock anything in.

## 3. Engine API (Rust-level)

The store is exposed as a single value, conventionally named `store`. All operations on the store are methods on this value. The store owns its scope tree and all transaction state; nothing about the store is global-static.

### 3.1 Lifecycle — construction and destruction

```rust
pub fn SemanticStore::new(config: SemanticStoreConfig) -> SemanticStore;     // Stable
```

- **Preconditions:** `config` validated by `SemanticStoreConfig::validate` (no zero-sized arena, no contradictory feature flags).
- **Postconditions:** an empty store at the root scope with no transactions in flight.
- **Errors:** none — construction is infallible after config validation.
- **Complexity:** O(1) excluding arena allocation; arena pre-allocation is bounded by `config.initial_capacity`.

```rust
impl Drop for SemanticStore { ... }                                          // Stable
```

- Frees the arena + all indexes deterministically. No facts or scopes persist after drop.

### 3.2 DECLARE (Stage 1) — fact-kind registration

```rust
pub fn store.declare_fact_kind(decl: FactKindDecl) -> Result<KindHandle, DeclareError>;  // Stable
```

- **Preconditions:** `decl` parsed from a grammar's `@fact_kind:` block and validated by `FactKindDecl::validate` (uniqueness in this store; every name in `required` / `indexes` appears in `attributes`; no zero-length index tuple).
- **Postconditions:** the kind is registered; index structures matching `decl.indexes` are allocated; the returned `KindHandle` is the canonical reference for all subsequent operations on this kind.
- **Errors:** `DeclareError::DuplicateKind { name }`, `DeclareError::UnknownAttributeInRequired { ... }`, `DeclareError::UnknownAttributeInIndex { ... }`, `DeclareError::EmptyIndex`.
- **Complexity:** O(number of indexes declared) for allocation; O(1) for the registry insertion.
- **Stability:** Stable.

```rust
pub struct FactKindDecl {                                                    // Stable
    pub name: String,
    pub attributes: Vec<String>,
    pub required: Vec<String>,
    pub indexes: Vec<Vec<String>>,
    pub scope_kind: Option<String>,
    pub exportable: bool,
    pub artefact_kind: Option<String>,
    pub description: Option<String>,
}
```

### 3.3 EMIT (Stage 2) — fact insertion

```rust
pub fn store.emit_fact(kind: KindHandle, attrs: AttrMap) -> Result<FactId, EmitError>;  // Stable
```

- **Preconditions:** `kind` from a successful `declare_fact_kind`; a transaction is active (or the store is in non-transactional mode); `attrs` carries every `required` attribute of the kind.
- **Postconditions:** a new `FactId` exists; the fact is in every index declared for the kind; the active transaction's undo log has one new entry; per-kind population counter has incremented.
- **Errors:** `EmitError::UnknownKind { handle }`, `EmitError::MissingRequired { kind, attr }`, `EmitError::AttributeNotDeclared { kind, attr }`, `EmitError::NoActiveScope` (only if `kind.scope_kind` requires one).
- **Complexity:** O(number of indexes for this kind). Effectively constant for typical 1–4 indexes per kind.
- **Stability:** Stable.

```rust
pub struct AttrMap { ... }                                                   // Stable
pub fn AttrMap::new() -> AttrMap;                                            // Stable
pub fn AttrMap::set<V: IntoValue>(&mut self, attr: &str, value: V);          // Stable
```

### 3.4 QUERY (Stage 3) — composable primitives

Four primitives, each O(1) average / O(log n) worst-case per §3.5 of the design.

```rust
pub fn store.has_fact(kind: KindHandle, name: &str) -> bool;                                  // Stable
pub fn store.fact_attribute(kind: KindHandle, name: &str, attr: &str) -> Option<&Value>;      // Stable
pub fn store.fact_count_at_least(kind: KindHandle, m: usize) -> bool;                         // Stable
pub fn store.resolve_path(dotted: &str) -> ResolveResult;                                     // Stable
```

- **`has_fact`** — preconditions: kind registered. Postconditions: returns true iff a fact of this kind with this name exists in any active scope or any imported library scope. Complexity: O(1) avg via the `(scope, kind, name)` index.
- **`fact_attribute`** — preconditions: kind registered; attr declared for the kind. Postconditions: returns the value if a fact exists and the attribute was supplied; returns None otherwise; never panics. Complexity: O(1) avg.
- **`fact_count_at_least`** — preconditions: kind registered. Postconditions: returns true iff the store contains at least M facts of this kind in any visible scope. Complexity: O(1) (counter-backed).
- **`resolve_path`** — preconditions: dotted name well-formed (matches `simple_identifier ( . simple_identifier )*`). Postconditions: returns `ResolveResult::Resolved { kind, attributes }` if every segment resolves to a fact, or `ResolveResult::Unresolved { last_resolved_segment_index, last_resolved_kind }` describing how far the walk got before failing. Complexity: O(number of segments) — linear in path depth, not store size.

```rust
pub enum ResolveResult {                                                     // Stable
    Resolved { kind: KindHandle, attributes: AttrMap },
    Unresolved { resolved_prefix: Vec<String>, last_kind: Option<KindHandle> },
}
```

Composed predicates (`@predicate_def`) are lowered to expressions over these four primitives at grammar-compile time and stored in the generated parser. They are never invoked through a separate engine API; they are inlined.

### 3.5 SCOPE (Stage 4) — open / close / introspect

```rust
pub fn store.open_scope(scope_kind: &str, name: &str) -> ScopeId;            // Stable
pub fn store.close_scope() -> ScopeId;                                       // Stable
pub fn store.current_scope() -> ScopeId;                                     // Stable
pub fn store.scope_chain(&self) -> impl Iterator<Item = ScopeId>;            // Stable (returns innermost-first)
```

- **`open_scope`** — preconditions: a transaction is active. Postconditions: pushes a new scope node as a child of the current scope; the new scope becomes current. Complexity: O(1).
- **`close_scope`** — preconditions: the current scope is not the root. Postconditions: pops the active chain; the closed scope persists in the scope tree (queryable via `scope_chain` on archived queries). Complexity: O(1). Triggers Stage 5 EXPORT for any fact-kind declared `exportable: true` whose facts live in the just-closed scope.
- **`scope_chain`** — preconditions: none. Postconditions: an iterator over scopes from innermost to outermost. Complexity: O(depth) to enumerate.

### 3.6 EXPORT (Stage 5) — library artefact write

```rust
pub fn store.export_scope(scope: ScopeId, lib_dir: &Path) -> Result<Vec<PathBuf>, ExportError>;  // Provisional
```

- **Preconditions:** scope exists in the tree (current or closed); `lib_dir` is writable.
- **Postconditions:** for every exportable fact-kind with facts in this scope, an atomically-written file at `lib_dir / <scope_kind> / <scope_name>.<artefact_kind>.facts.json` containing the facts in the documented Stable artefact format. Returned vector lists every file written.
- **Errors:** `ExportError::Io { ... }`, `ExportError::SerializeFailed { kind, name, ... }`, `ExportError::ScopeNotExportable { reason }`.
- **Complexity:** O(facts exported); I/O bounded.
- **Stability:** Provisional — the file-naming convention and the JSON envelope are documented in `PGEN_SEMANTIC_STORE_LIBRARY_ARTEFACT_FORMAT.md` (separate doc, to be drafted in Phase 0.5).

This is typically invoked **automatically** by `close_scope` for fact-kinds with `exportable: true`. Direct invocation is exposed for tooling (e.g., `pgen export-library`).

### 3.7 IMPORT (Stage 6) — library artefact read (lazy)

```rust
pub fn store.import_from_library(kind: &str, name: &str, lib_dir: &Path) -> Result<ImportHandle, ImportError>;  // Stable
```

- **Preconditions:** a transaction is active; `lib_dir` is readable; `<lib_dir>/<kind>/<name>.facts.json` exists.
- **Postconditions:** the artefact's index entries are eagerly loaded into the current scope; attribute values are loaded lazily on first query. Imported facts are visible to queries but marked as imported (`Fact::source == Source::Imported { lib_dir, kind, name }`); they are not re-exported unless re-emitted locally.
- **Errors:** `ImportError::ArtefactNotFound { ... }`, `ImportError::FormatVersionUnsupported { found, supported }`, `ImportError::DeserializeFailed { ... }`, `ImportError::SchemaConflict { ... }` (if the imported artefact declares a kind the local schema doesn't recognise — proposed handling: `[TBC]`).
- **Complexity:** O(index entries in the artefact) for the eager load; O(1) per subsequent lazy attribute fetch.
- **Stability:** Stable.

### 3.8 ROLLBACK (Stage 7) — transactions

```rust
pub fn store.begin_transaction() -> TxId;                                    // Stable
pub fn store.commit_transaction(tx: TxId) -> Result<(), TxError>;            // Stable
pub fn store.rollback_transaction(tx: TxId) -> Result<(), TxError>;          // Stable
pub fn store.current_transaction() -> Option<TxId>;                          // Stable
```

- **`begin_transaction`** — preconditions: none (transactions nest). Postconditions: a new transaction is the active one; the parent transaction (if any) is suspended; rollback within this transaction unwinds only operations performed within this transaction. Complexity: O(1).
- **`commit_transaction(tx)`** — preconditions: `tx` is the current transaction. Postconditions: this transaction's undo log is **merged into the parent transaction's undo log** (or discarded if this was the top-level transaction). Parent transaction (if any) becomes active. Complexity: O(undo-log entries in committed tx) when merging; O(1) at top level.
- **`rollback_transaction(tx)`** — preconditions: `tx` is the current transaction. Postconditions: every operation logged in this transaction's undo log is reversed (facts removed from every index they were inserted into; scope opens popped; lazy imports unloaded). Parent transaction (if any) becomes active. **Guarantee:** the store is byte-identical to its state at `begin_transaction(tx)` time. Complexity: O(undo-log entries) — never O(store).
- **Errors:** `TxError::NotCurrentTransaction { tx, current }`, `TxError::AlreadyCommitted { tx }`, `TxError::AlreadyRolledBack { tx }`.

Transactions are the **only** mechanism for speculative parsing. The generator-emitted `with_semantic_runtime_rule_transaction` IIFE (from `.3.3.3`) is the canonical entry point and ensures commit-on-Ok / rollback-on-Err discipline.

### 3.9 Diagnostics and observability

```rust
pub fn store.counters(&self) -> Counters;                                    // Provisional
pub fn store.explain(&self, query: ExplainQuery) -> ExplainPlan;             // Provisional
pub fn store.dump_facts(&self, filter: DumpFilter) -> impl Iterator<Item = FactView>;  // Provisional
```

- **`counters`** — preconditions: none. Postconditions: returns a snapshot of per-primitive operation counters (emits, queries, rollbacks per kind, scope opens, library loads). Used by `--explain`, perf benches, and observability hooks. Complexity: O(number of kinds + number of indexes) for the snapshot.
- **`explain(query)`** — preconditions: query well-formed. Postconditions: returns which index would serve this query, expected scan size, projected wall-clock cost. Does not execute the query (read-only plan). Complexity: O(1) per query primitive considered.
- **`dump_facts(filter)`** — preconditions: filter well-formed. Postconditions: yields a `FactView` per matching fact, including `(scope, kind, name, attributes, source)`. Used by `pgen dump-library` and SEMTRACE. Complexity: O(matching facts) — iteration only over matched, not over all.

All three are `Provisional` because the precise field set in `Counters` / `ExplainPlan` / `FactView` will likely evolve as we add new diagnostics; the existence of the surfaces is stable.

### 3.10 Configuration

```rust
pub struct SemanticStoreConfig {                                             // Provisional
    pub initial_capacity: usize,           // for the fact arena
    pub max_undo_log_per_tx: Option<usize>,// safety cap on undo-log growth, None = unbounded
    pub artefact_format_version: u32,      // for library export
    pub strict_schema_conflicts: bool,     // see ImportError::SchemaConflict
    pub trace_hook: Option<TraceHook>,     // for PGEN_TRACE_VERBOSITY integration
}
```

## 4. Annotation surface (grammar-author-level)

The annotation language extensions live in `grammars/semantic_annotation.ebnf` (to be specified precisely in artefact 2 of Phase 0.5). The annotation forms below are the **shapes** the grammar will accept; the EBNF specifying their grammar is the companion artefact.

### 4.1 DECLARE — `@fact_kind:`

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

- **Where it appears:** at the top of a grammar, before any rule that emits or queries this kind.
- **Validation:** at grammar-compile time (codegen): uniqueness across the grammar; every name in `required`/`indexes` appears in `attributes`; no zero-length index tuple; `scope_kind` must be one of the labels used in this grammar's `@open_scope`.
- **Stability:** Stable.

### 4.2 EMIT — `@emit_fact:`

```ebnf
@emit_fact: { kind: variable_binding,
              name: $variable_name,
              type_kind: $resolved_type_kind,
              type_ref: $type_ref_body,
              declared_in: $current_scope_label }
some_decl_rule := ...
```

- **Value expressions** in attribute slots accept the existing semantic-annotation language: positional captures (`$N`), named captures (`$name`), dotted property access (`$x.body`), indexed access (`$x[0]`), and scalar literals.
- **Validation:** at grammar-compile time, `kind` must reference a declared `@fact_kind`; supplied attribute names must be a subset of the kind's `attributes`; missing-required is a parse-time error (not compile-time, since values are evaluated at runtime).
- **Fires:** on rule commit (post body parse, pre return-shape construction).
- **Stability:** Stable.

### 4.3 QUERY — `@predicate` (primitive form)

```ebnf
@predicate has_fact args:[variable_binding, $variable_name] phase: branch
@predicate fact_attribute_equals args:[variable_binding, $variable_name, type_kind, array] phase: post
@predicate fact_count_at_least args:[variable_binding, 1] phase: pre
@predicate resolve_path args:[$dotted_receiver] phase: branch
```

- **Phases:** `pre` (before rule body), `branch` (inside alternation, gates which branch fires), `post` (after rule body, before commit).
- **Stability:** Stable.

### 4.4 QUERY — `@predicate_def:` (composed form)

```ebnf
@predicate_def: {
  name: receiver_is_array,
  args: [receiver_path],
  body: resolve_path($receiver_path).attribute("type_kind") in ["array", "queue", "dynamic_array", "assoc_array"]
}
```

then used as:
```ebnf
@predicate receiver_is_array args:[$enclosing_receiver] phase: branch
```

- **Body language:** expression over the four primitives, with `.attribute("name")` lookup, `in [literal, ...]` membership, `==`/`!=`/`&&`/`||`/`!` logical operators. Full EBNF in the schema-language spec (artefact 2 of Phase 0.5).
- **Validation:** at grammar-compile time, the body parses; all primitives invoked exist; all arg references are bound.
- **Stability:** Stable.

### 4.5 SCOPE — `@open_scope` / `@close_scope`

```ebnf
@open_scope: { kind: class, name: $class_name }
class_declaration := ... @close_scope
```

- **Stability:** Stable.

### 4.6 IMPORT — `@import_from_library`

```ebnf
@import_from_library: { kind: bindings, name: $package_name }
package_import_item := ...
```

- **Stability:** Stable.

## 5. Error taxonomy

All errors crossing the engine API boundary are typed enums implementing `std::error::Error`. Categories:

- `DeclareError` — Stage 1 failures (kind registration).
- `EmitError` — Stage 2 failures (fact insertion).
- `QueryError` — never returned for the four primitives (they return `bool` / `Option`); reserved for future query forms with richer failure modes.
- `ScopeError` — Stage 4 failures (close-without-open, etc.).
- `ExportError` — Stage 5 failures (I/O, serialisation).
- `ImportError` — Stage 6 failures (not-found, version mismatch, deserialisation).
- `TxError` — Stage 7 failures (transaction misuse).
- `SemanticStoreError` — top-level enum wrapping all of the above (for `From` conversions in callers).

Every variant carries enough context for a precise error message (which kind, which attribute, which file, etc.). Engine code **never panics on user input**; panics are reserved for genuine internal-consistency bugs.

## 6. Complexity contracts — single table

| Operation | Complexity (avg / worst) | Bound source |
|---|---|---|
| `declare_fact_kind` | O(indexes) | per-kind allocation |
| `emit_fact` | O(indexes) / O(indexes log n) | one insert per index |
| `has_fact` | O(1) / O(log n) | hash-indexed `(scope, kind, name)` |
| `fact_attribute` | O(1) / O(log n) | hash-indexed |
| `fact_count_at_least` | O(1) / O(1) | counter |
| `resolve_path` | O(depth) / O(depth log n) | per-segment lookup |
| `open_scope` | O(1) / O(1) | arena allocation |
| `close_scope` | O(1) + O(exportable facts in scope) for triggered export | linked-list pop + optional export |
| `export_scope` | O(facts exported) + I/O | serialisation |
| `import_from_library` | O(index entries in artefact) | lazy load |
| `begin_transaction` | O(1) / O(1) | undo-log header |
| `commit_transaction` | O(undo entries) — merge to parent | merge |
| `rollback_transaction` | O(undo entries) | reverse each op |
| `counters` | O(kinds + indexes) | snapshot |
| `explain` | O(1) per primitive considered | plan only |
| `dump_facts` | O(matching) | iteration |

**The performance contract in artefact 4 of Phase 0.5 binds these to concrete latency numbers on a baseline machine.**

## 7. Open questions (for review)

- **`[TBC]`** Should `emit_fact` accept structured `AttrMap` (typed) or untyped `serde_json::Value`? Tradeoff: typed = compile-time safety + faster; untyped = simpler API + serde-friendly. Current sketch leans typed.
- **`[TBC]`** Should `resolve_path` traverse only the active scope chain, or also archived (closed) scopes? §3 of the design says "scope tree, not stack", which implies closed-scope walkability — but for path resolution specifically that may be surprising. Lean: active chain + parent chain only; closed-scope reads only via explicit `store.scope(id).query(...)`.
- **`[TBC]`** Library artefact format — JSON now (continuity with `.3.3.4.a` MVP-0), but should we plan for binary (e.g., FlatBuffers / Cap'n Proto) at 1.0.0? JSON is human-readable and migration-friendly; binary is faster to parse. Probably JSON now, binary as an opt-in in 1.1.0.
- **`[TBC]`** Should `import_from_library` fully load the artefact eagerly when `--lib-eager` is set, for benchmarking purposes? (Lazy is the production default.)
- **`[TBC]`** `SchemaConflict` policy in `import_from_library` — if an artefact declares a kind we don't know about, do we error (strict), drop the unknown kind (lenient), or expose it as an opaque kind queryable by name only (passthrough)? Current sketch: strict, but `config.strict_schema_conflicts = false` enables lenient.

## 8. Glossary

- **Fact** — one (kind, name, attributes) tuple stored at one scope.
- **Fact-kind** — a declared family of facts sharing the same attributes / indexes / scope discipline.
- **Index** — a data structure keyed on some subset of fact attributes, enabling O(1) avg lookup for that key shape.
- **Scope** — a node in the scope tree, identified by `(kind, name)`. Scopes are created by `@open_scope`, retired by `@close_scope`, and persist in the tree for archived queries.
- **Transaction** — a unit of speculative parsing. Operations within a transaction are atomically committed or rolled back. Nested transactions compose.
- **Library artefact** — a serialised snapshot of all exportable facts in a closed scope, persisted as a file under `<lib-dir>`.

## 9. References

- `docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md` — the design this contract implements.
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` §1.0.122 — the `.3.3.4.a` MVP-0 library mechanism; this contract extends it.
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` §1.0.121 — the `.3.3.3` IIFE exception-safety pattern; the canonical transaction wrapper this contract assumes.
- `rust/src/ast_pipeline/semantic_runtime.rs` — current home of the (pre-this-contract) `SemanticRuntimeState`; will be refactored / extended to implement this contract.
- `feedback_universal_semantic_store` (memory) — principle of universality + sign-off quality + lifecycle protocol.

---

**Amend freely.** The sketch is the starting point for the review pass; everywhere you see `Provisional` or `[TBC]` is up for revision. Once reviewed and accepted, the document is renamed from `Sketch` → `Draft 1.0.0-rc` and the Stable markers become binding.
