# PGEN Semantic Store — Test Plan

**Status:** DRAFT (sketch — first cut for review).
**Companion to:** `docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md` (semantics) and `docs/contracts/PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md` (surface syntax).
**Audience:** Phase 1 implementor; CI gate authors; reviewers.

---

## 1. Purpose

Enumerate the test cases that must be GREEN before any code lands in the semantic store subsystem. Every test case binds a specific contractual element from the API contract or the schema spec, and identifies the test kind (unit / property / stress / adversarial / end-to-end).

The test plan is the **falsification surface** for the design. If we can't write a test case that proves a behaviour, the behaviour isn't specified clearly enough.

## 2. Test categories

- **U — Unit** — per-primitive behaviour with hand-written fixtures. Fast (<1ms each). Run on every commit.
- **P — Property** — invariants checked via randomized inputs (proptest or quickcheck). Medium (<100ms each). Run on every commit.
- **S — Stress** — scale tests with ≥1M facts / ≥100k scopes. Slow (seconds). Run nightly + on release.
- **A — Adversarial** — malformed inputs, attempted misuse, edge cases at boundaries. Fast. Run on every commit.
- **E — End-to-end** — synthetic grammars exercising the full lifecycle. Medium. Run on every commit.

## 3. Coverage matrix

Every public API primitive (§3 of the API contract) and every validation rule (V-*) in the schema spec must have at least one test case.

| Surface | Test cases below |
|---|---|
| `SemanticStore::new` / `Drop` | §4.1 |
| `declare_fact_kind` + V-DECL-1..7 | §4.2 |
| `emit_fact` + V-EMIT-1..4 | §4.3 |
| `has_fact` / `fact_attribute` / `fact_count_at_least` / `resolve_path` | §4.4 |
| `open_scope` / `close_scope` / `current_scope` / `scope_chain` | §4.5 |
| `export_scope` (+ format-stability) | §4.6 |
| `import_from_library` (+ lazy semantics) | §4.7 |
| `begin_transaction` / `commit_transaction` / `rollback_transaction` | §4.8 |
| `counters` / `explain` / `dump_facts` | §4.9 |
| Schema spec V-rules (V-DECL / V-EMIT / V-QPRIM / V-QDEF / V-SCOPE / V-IMPORT) | §4.10 |
| End-to-end synthetic grammar | §5 |
| Stress + perf regression | §6 |
| Adversarial | §7 |

## 4. Unit + property tests (per primitive)

### 4.1 Construction / destruction

- **U-CTOR-1** — `new` with valid config returns a store at root scope with zero facts, zero transactions, zero declared kinds.
- **U-CTOR-2** — `new` with `initial_capacity: 0` returns Err (invalid config).
- **U-DROP-1** — dropping a store with active transactions, open scopes, declared kinds, and N facts leaks nothing (verified via leak-sanitiser CI).
- **P-CTOR-1** — for any well-formed config, `new().drop()` is a no-op-equivalent (no I/O, no panics, no leaks).

### 4.2 DECLARE — `declare_fact_kind` + V-DECL rules

- **U-DECL-1** — declaring a well-formed kind returns `Ok(KindHandle)`; the handle uniquely identifies this kind.
- **U-DECL-2 (V-DECL-1)** — declaring two kinds with the same name returns `Err(DuplicateKind)` on the second.
- **U-DECL-3 (V-DECL-2)** — declaring with `attributes: []` returns `Err(EmptyAttributes)`.
- **U-DECL-4 (V-DECL-3)** — declaring with `required: [missing_attr]` returns `Err(UnknownAttributeInRequired)`.
- **U-DECL-5 (V-DECL-4)** — declaring with `indexes: [(missing_attr,)]` returns `Err(UnknownAttributeInIndex)`.
- **U-DECL-6 (V-DECL-5)** — declaring with `indexes: [(name, name)]` (duplicate within tuple) returns `Err(DuplicateInIndexTuple)`.
- **U-DECL-7 (V-DECL-5)** — declaring with `indexes: [()]` (empty tuple) returns `Err(EmptyIndexTuple)`.
- **U-DECL-8 (V-DECL-6)** — declaring with `scope_kind: nonexistent` returns `Err(UnknownScopeKind)` (or warning if config.lenient_scope_kinds).
- **U-DECL-9 (V-DECL-7)** — declaring with `name: ../foo` returns `Err(InvalidPathComponent)`.
- **P-DECL-1** — for any well-formed `FactKindDecl`, `declare_fact_kind` succeeds and the kind is queryable by name immediately.
- **P-DECL-2** — for any malformed `FactKindDecl`, `declare_fact_kind` errors and the store's set of declared kinds is unchanged.

### 4.3 EMIT — `emit_fact` + V-EMIT rules

- **U-EMIT-1** — emitting a well-formed fact returns `Ok(FactId)`; the fact is queryable via `has_fact` immediately.
- **U-EMIT-2 (V-EMIT-2)** — emitting with `kind: undeclared` returns `Err(UnknownKind)`.
- **U-EMIT-3 (V-EMIT-3)** — emitting with `attr: not_in_attributes` returns `Err(AttributeNotDeclared)`.
- **U-EMIT-4 (V-EMIT-4)** — emitting without a `required` attribute returns `Err(MissingRequired)` with `{kind, attr}` in the error.
- **U-EMIT-5** — emitting outside a transaction (when transactional mode is active) returns `Err(NoActiveTransaction)`.
- **U-EMIT-6** — emitting into a kind with `scope_kind: enclosing_class` while no class scope is open returns `Err(NoActiveScope { expected_kind: class })`.
- **U-EMIT-7** — emitting two facts with the same `(scope, kind, name)` — what happens? **`[TBC]`** Lean: second emit returns `Err(DuplicateInScope)`; alternative: replace + warning. Design needs to pick.
- **P-EMIT-1** — for any K declared and any well-formed attrs, `emit_fact` succeeds; immediately after, `has_fact(kind, attrs.name)` returns true.
- **P-EMIT-2** — for any K declared with `indexes: [I1, I2, ...]`, `emit_fact` inserts into every Ii; a query against each Ii after emit retrieves the fact.

### 4.4 QUERY — four primitives

#### has_fact

- **U-QUERY-HAS-1** — `has_fact` for an existing fact returns true.
- **U-QUERY-HAS-2** — `has_fact` for a non-existent fact returns false.
- **U-QUERY-HAS-3** — `has_fact` for a fact in a closed scope (not in active chain) returns false.
- **U-QUERY-HAS-4** — `has_fact` for an imported fact returns true.
- **P-QUERY-HAS-1** — `has_fact(K, N)` is true iff a fact `(K, N)` was emitted in any active or imported scope and not rolled back or unimported.

#### fact_attribute

- **U-QUERY-ATTR-1** — `fact_attribute` for an existing fact + declared attr returns Some(value).
- **U-QUERY-ATTR-2** — `fact_attribute` for an existing fact + missing-from-emit attr returns None.
- **U-QUERY-ATTR-3** — `fact_attribute` for an undeclared attr returns None (never panics).
- **U-QUERY-ATTR-4** — `fact_attribute` for a non-existent fact returns None.

#### fact_count_at_least

- **U-QUERY-COUNT-1** — `fact_count_at_least(K, 0)` always true.
- **U-QUERY-COUNT-2** — `fact_count_at_least(K, N+1)` after emitting N facts of K is false; after emitting one more is true.
- **U-QUERY-COUNT-3** — counter respects rollback: emit 5, rollback 3 → count_at_least(K, 3) true, count_at_least(K, 5) false.

#### resolve_path

- **U-QUERY-RESOLVE-1** — `resolve_path("simple_name")` for an existing top-level fact returns `Resolved`.
- **U-QUERY-RESOLVE-2** — `resolve_path("a.b")` for `a = class instance, a.b = field of class` returns `Resolved`.
- **U-QUERY-RESOLVE-3** — `resolve_path("a.b.c")` traverses through a + b's class-member scope to find c.
- **U-QUERY-RESOLVE-4** — `resolve_path("a.nonexistent")` returns `Unresolved { resolved_prefix: ["a"], last_kind: Some(class) }`.
- **U-QUERY-RESOLVE-5** — `resolve_path("")` returns `Err(EmptyPath)` (or Unresolved with empty prefix — design pick).
- **P-QUERY-RESOLVE-1** — for any sequence of `emit_fact` calls producing a chain of class → member → ..., `resolve_path` walks the chain correctly.

### 4.5 SCOPE — open / close / chain

- **U-SCOPE-1** — `open_scope` pushes; `current_scope` reflects it.
- **U-SCOPE-2** — `close_scope` pops; `current_scope` reflects the parent.
- **U-SCOPE-3** — `close_scope` at root returns `Err(CannotCloseRoot)`.
- **U-SCOPE-4** — `scope_chain` returns innermost-to-outermost.
- **U-SCOPE-5** — facts emitted in inner scope are queryable from inner; queryable from outer iff scope_kind allows (depends on default-scope semantics).
- **U-SCOPE-6** — closed scope persists in the tree; `dump_facts` filtered on the closed scope still returns its facts.
- **P-SCOPE-1** — for any nested open/close sequence, the scope chain's depth equals (opens - closes).

### 4.6 EXPORT — library write

- **U-EXPORT-1** — `export_scope` for a scope with exportable facts writes the expected JSON file at `<lib-dir>/<scope_kind>/<scope_name>.<artefact_kind>.facts.json`.
- **U-EXPORT-2** — exported JSON contains every exportable fact with all attributes; non-exportable kinds are skipped.
- **U-EXPORT-3** — atomic write — interruption mid-write leaves no partial file (verified by simulated kill at fsync boundary).
- **U-EXPORT-4** — `format_version` field is present and matches the configured version.
- **U-EXPORT-5** — `close_scope` triggers automatic export when at least one exportable kind has facts; if none have facts, no file is written.
- **P-EXPORT-1** — for any scope S with N exportable facts, `export_scope(S, dir)` followed by `import_from_library(...)` round-trips: query results identical post-import.

### 4.7 IMPORT — library read (lazy)

- **U-IMPORT-1** — `import_from_library` for an existing artefact returns `Ok(ImportHandle)`; facts queryable immediately via `has_fact`.
- **U-IMPORT-2** — `import_from_library` for a missing file returns `Err(ArtefactNotFound)`.
- **U-IMPORT-3** — `import_from_library` for an artefact with `format_version` newer than supported returns `Err(FormatVersionUnsupported)`.
- **U-IMPORT-4** — lazy attribute load: emitting `M` facts to a library, importing, then querying `fact_attribute` for fact #M should NOT have loaded fact #0..M-1's attributes (verified via counter `lazy_loads_performed`).
- **U-IMPORT-5** — imported facts have `source: Imported { lib_dir, kind, name }`; they are filterable in `dump_facts`.
- **U-IMPORT-6** — re-exporting after import does not re-include imported facts (they must be locally re-emitted to be re-exported).

### 4.8 TRANSACTIONS — begin / commit / rollback

- **U-TX-1** — `begin_transaction` returns a fresh TxId; `current_transaction` reflects it.
- **U-TX-2** — `commit_transaction` for the current Tx returns Ok; current tx is now the parent (None if top-level).
- **U-TX-3** — `rollback_transaction` for the current Tx undoes every operation since `begin`; current tx is now the parent.
- **U-TX-4** — emitting 5 facts within a Tx then rolling back leaves 0 visible facts; emit + commit leaves 5.
- **U-TX-5** — opening 3 scopes within a Tx then rolling back reverts the scope chain to its pre-Tx state.
- **U-TX-6** — importing a library within a Tx then rolling back unloads the import.
- **U-TX-7** — nested Tx: begin outer, begin inner, emit 5 in inner, commit inner, rollback outer ⇒ 0 facts (inner's commit folded into outer's undo log, then unwound).
- **U-TX-8** — nested Tx: begin outer, begin inner, emit 5 in inner, rollback inner, commit outer ⇒ 0 facts (inner's emits never made it to outer).
- **U-TX-9** — `commit_transaction(tx)` for a non-current Tx returns `Err(NotCurrentTransaction)`.
- **U-TX-10** — `rollback_transaction(tx)` for a non-current Tx returns `Err(NotCurrentTransaction)`.
- **P-TX-1 (THE KEY INVARIANT)** — for any sequence of operations (emit / scope / import) inside a `begin_transaction(); ...; rollback_transaction()` block, the store is byte-identical to its pre-`begin` state. Verified by checksumming all indexes and the scope tree.
- **P-TX-2** — for any sequence of operations + a final commit, the store reflects exactly the operations performed (no double-application, no skipped operations).
- **S-TX-1** — emit 1M facts in one transaction, rollback, measure: rollback completes in O(M) time (target: < 1s for 1M facts on baseline hardware; performance contract numerizes).

### 4.9 DIAGNOSTICS — counters / explain / dump

- **U-COUNTERS-1** — counters reflect operation counts accurately: emit 100 facts of kind K → `counters.emits[K] == 100`.
- **U-COUNTERS-2** — counters survive rollback (they count operations attempted, not committed) **`[TBC]`**: or do they reset on rollback? Design pick.
- **U-EXPLAIN-1** — `explain(has_fact(K, N))` returns plan = "use index `(scope, kind, name)`; scan size 1; cost O(1)".
- **U-DUMP-1** — `dump_facts(filter: ByKind(K))` yields every fact of kind K in any visible scope.
- **U-DUMP-2** — `dump_facts` with no filter yields every fact in the store.

### 4.10 Schema spec validation rules (V-*)

Each V-* rule from the schema spec needs a test case. Already covered in §4.2-§4.9 above for V-DECL-1..7, V-EMIT-1..4; below are the remaining:

- **V-QPRIM-1** — predicate name resolution (built-in or user-defined).
- **V-QPRIM-2** — predicate args arity.
- **V-QPRIM-3** — phase validity.
- **V-QDEF-1** — composed predicate name uniqueness.
- **V-QDEF-2** — composed predicate args distinct.
- **V-QDEF-3** — primitive calls in body arity correct.
- **V-QDEF-4** — `$arg` references are bound.
- **V-QDEF-5** — `.attribute()` only on fact-returning primitives.
- **V-SCOPE-1** — `kind` non-empty.
- **V-SCOPE-2** — close balances open.
- **V-SCOPE-3** — scope_kind labels referenced.
- **V-IMPORT-1** — artefact kind known.

Each of these is a **codegen-time test** — feed a malformed grammar to the codegen and assert a precise diagnostic comes out. Land in `rust/tests/semantic_store_schema_validation.rs`.

## 5. End-to-end synthetic grammar (E-tests)

Build a tiny grammar (`grammars/synthetic_semantic_store_smoke.ebnf`) exercising every lifecycle stage:

- One `@fact_kind` declaration for a `type_binding` kind.
- One `@fact_kind` declaration for a `variable_binding` kind with multiple indexes.
- One `@predicate_def` composing primitives.
- Rules with `@open_scope`/`@close_scope` for a `class` scope kind.
- Rules with `@emit_fact` for both kinds.
- A rule with `@predicate has_fact phase: post` gating commit.
- A rule with `@predicate_def`-defined predicate gating a branch (`phase: branch`).
- Rules with `@import_from_library` for a `bindings` library.

Tests:

- **E-LIFECYCLE-1** — parse a sample input through the grammar; verify facts emitted, scopes opened/closed, predicates fired, and final state matches expectations.
- **E-LIFECYCLE-2** — round-trip: parse input, export, then re-parse a separate input importing the export. Cross-file facts visible at the importer.
- **E-LIFECYCLE-3** — speculative parse with rollback: parse input that triggers a try_parse path with emits, verify rollback reverts everything.
- **E-LIFECYCLE-4** — `--explain` output for a representative predicate query is non-empty and references the expected index.
- **E-LIFECYCLE-5** — `pgen dump-library <artefact>` produces a human-readable view of all facts.

## 6. Stress + performance regression (S-tests)

Bench harness lands in `rust/benches/semantic_store.rs`.

- **S-PERF-EMIT-1** — emit 1M facts; verify total time scales linearly; assert mean emit time within budget (artefact 4 sets the number; sketch: ≤500ns per emit on baseline hardware).
- **S-PERF-HAS-FACT-1** — at 1M facts, `has_fact` p99 latency within budget (sketch: ≤200ns).
- **S-PERF-RESOLVE-PATH-1** — at 1M facts + 100k scopes, resolve a depth-5 path; p99 latency within budget (sketch: ≤2µs).
- **S-PERF-ROLLBACK-1** — at 1M facts in one Tx, rollback; p99 latency within budget (sketch: ≤1µs per emitted fact = ≤1s total).
- **S-PERF-EXPORT-1** — export a scope with 100k facts; verify atomic + within budget (sketch: ≤500ms).
- **S-PERF-IMPORT-LAZY-1** — import a 50MB artefact; verify only index entries loaded; per-attribute lookup measured separately.
- **S-MEMORY-1** — 1M facts + 100k scopes; verify total memory within budget (sketch: ≤500 bytes per fact, ≤200 bytes per scope = ≤520 MB total; performance contract specifies).
- **S-SCALABILITY-1** — emit / query / rollback latencies at 10k / 100k / 1M / 10M facts; verify scaling is O(1) avg per primitive (no slope as N grows).

CI gate: any S-test regression > 10% from the contract baseline blocks merge.

## 7. Adversarial tests (A-tests)

- **A-MALFORMED-FACT-KIND-1** — feed `@fact_kind` with binary garbage in a field; expect clean error, no panic.
- **A-RECURSIVE-PREDICATE-1** — `@predicate_def: { name: foo, body: foo($x) }` — expect cycle-detection error at codegen.
- **A-SCHEMA-CONFLICT-1** — load a library artefact with a kind name that collides with a locally-declared kind with different attributes — expect `Err(SchemaConflict)` (in strict mode) or warning + use-local (in lenient mode).
- **A-CORRUPT-ARTEFACT-1** — truncated JSON file → `Err(DeserializeFailed)` with file path + offset.
- **A-INFINITE-SCOPE-1** — open 1M scopes without closing — verify no stack overflow (scope tree is arena-allocated, not stack-recursed).
- **A-RAPID-TX-1** — begin/rollback 100k times in a tight loop — verify no memory leak (counter delta zero post-loop).
- **A-CONCURRENT-MUTATION-1** — `emit_fact` while iterating `dump_facts` — Rust's borrow checker should prevent this at compile time; verified by `cargo check` in tests/.
- **A-NAME-WITH-NULL-1** — fact name containing `\0` byte — should reject at emit time (sketch lean: yes).
- **A-EMPTY-STORE-1** — every query primitive on an empty store returns "no match" not "error".

## 8. CI integration

- All U-, A-, E- tests run on every push (target: < 30s total).
- All P- tests run on every push (target: < 2 min total; budget for proptest shrinking).
- All S- tests run nightly + on release-candidate branches (target: < 15 min total).
- Test results published as a build artefact; perf-bench results landed in `target/criterion/...` and tracked against the performance contract baseline.

## 9. Acceptance criteria

This test plan is considered satisfied when:

1. Every test ID listed above has a corresponding test in `rust/tests/` or `rust/benches/`.
2. Every test passes in CI on a baseline machine.
3. Every V-* rule from the schema spec has at least one test case.
4. The end-to-end synthetic grammar exercises every lifecycle stage (DECLARE / EMIT / QUERY / SCOPE / EXPORT / IMPORT / ROLLBACK).
5. The stress tests at 1M facts / 100k scopes meet the performance contract numbers.
6. No `#[should_panic]` test exists for any user-input failure mode — all such cases return typed errors.

## 10. Open questions

- **`[TBC]`** Duplicate-in-scope (`U-EMIT-7`): error or replace? Lean: error in strict mode; warn+replace in lenient.
- **`[TBC]`** Counter reset on rollback (`U-COUNTERS-2`): rollback resets counters or counters track attempts? Lean: counters track ATTEMPTS (so observability shows speculative-parse churn).
- **`[TBC]`** `resolve_path("")` (`U-QUERY-RESOLVE-5`): error or empty-resolved? Lean: error.
- **`[TBC]`** Should there be a test for the `--explain` HUMAN-READABILITY of the plan output, or only its structural correctness? Sketch: structural only; HR is a UX concern.

## 11. References

- `docs/contracts/PGEN_SEMANTIC_STORE_API_CONTRACT.md` — every U-/P- test binds to a primitive specified there.
- `docs/contracts/PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md` — every V-* rule has a corresponding test.
- `docs/contracts/PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md` (artefact 4 — pending) — sets the concrete numbers the S-tests assert against.
- `docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md` — the design driving all of this.

---

**Amend freely.** Test names follow `<CATEGORY>-<AREA>-<NUMBER>` (e.g., `U-DECL-3`, `P-TX-1`, `S-PERF-EMIT-1`) so reviewers can reference any case unambiguously.
