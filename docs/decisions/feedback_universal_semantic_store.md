<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_universal_semantic_store.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_universal_semantic_store
description: "The parser's semantic store is a SIGN-OFF-QUALITY universal/schema-agnostic database — any information encountered during parsing storable; engine never enumerates known fact-kinds; multi-index by query shape; new fact-kinds added by declaration (zero engine churn); designed and built with care like an IC heading to tape-out, because it stays with us for a long time."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

User-stated principle, 2026-05-21 (during `.3.3.4.b.4` design review):

> "Any information encountered during parsing should be stored, organised using various techniques so that any kind of queries issued or questions asked by the parser later on can be quickly and efficiently answered in order for it to navigate the EBNF maze when digesting the input file."

Reinforced + escalated 2026-05-21 (same session):

> "The semantic-store shall be organized as a database for very fast retrieving based on various and extensible criteria. It should be like a very, very well organized memory where it is cheap and fast to save and very cheap and fast to restore or retrieve. The semantic-store shall by no means be the bottleneck of the parser. I do not know the types of the semantic information we might need to save and restore in the future, but there is one thing I am sure about — it won't cost us anything to describe this new semantic info type, save it, retrieve it. The semantic-store shall be top-notch, sign-off level quality. It shall be designed and built with a lot of care, because it is going to stay with us for a long, long time, it better be very good."

**Quality bar (sign-off level, per `CONTEXT_AWARE_PARSING_DESIGN.md` §10):**
- Phase 0.5 (`.3.3.4.b.5.0`) is design-before-code: API contract + schema-language spec + test plan + perf contract land FIRST, reviewed, before any code touches the engine.
- Performance budgets are explicit: `has_fact` ≤200ns p99 at 1M facts; rollback ≤1µs/emitted-fact; library import lazy ≤10ms cold-start; no quadratic anywhere on the hot path.
- Test discipline: unit tests + property tests (insert-then-lookup, rollback-leaves-no-trace, scope-walk-correctness) + stress tests (≥1M facts, ≥100k scopes) + adversarial tests + perf benches as continuous CI gates.
- API stability: public surface versioned in `PGEN_SEMANTIC_STORE_API_CONTRACT.md`; breaking changes require deprecation + migration cookbook + major version bump.
- Observability: per-primitive + per-index + per-kind counters; `--explain` mode for predicate queries; library-artefact dump tool. Hooked into `PGEN_TRACE_VERBOSITY`.
- Library artefact format is versioned + forward-readable + has a `pgen migrate-library` tool.
- Sign-off means: API contract reviewed; 100% public surface unit-tested + property-tested; complexity + error semantics documented; perf benches within budget on a baseline machine; schema-language exercised end-to-end by a synthetic grammar (Phase 1) and SV (Phase 4); at least one migration scripted to prove the mechanism.

**Lifecycle protocol (`CONTEXT_AWARE_PARSING_DESIGN.md` §4, added 2026-05-21):**

User mandate, same session:

> "We need a standard, very efficient and systematic way of describing a new semantic fact to be added to the store, adding an instance of this new type to the store and retrieving such instance from the store or globally — and interactions with the store related to a semantic-annotation type (static) and life (runtime, parse time, collection, addition, retrieval) shall be systematised rigorously, no place for improvisation."
>
> "It shall be flexible and extensible so that it can deal with any semantic type we can think of."

Every fact-kind walks the same SEVEN stages:

1. **DECLARE** (compile-time, `@fact_kind: {…}` block) — kind name, attributes, requireds, indexes, scope_kind, exportable, artefact_kind, description. Validation at codegen.
2. **EMIT** (parse-time, `@emit_fact { kind: K, … }` on rule) — on rule commit: validate kind, validate requireds, evaluate attribute exprs, determine scope, insert into every index, undo-log.
3. **QUERY** (parse-time, `@predicate <name>(…)` on rule) — composable primitives (`has_fact`, `fact_attribute_equals`, `fact_count_at_least`, `resolve_path`) plus named composed predicates (`@predicate_def: {…}`). Phases `pre`/`branch`/`post`.
4. **SCOPE** (parse-time, `@open_scope { kind, name }` / `@close_scope`) — scope tree (not just stack); closed scopes stay queryable.
5. **EXPORT** (parse-time, automatic when fact-kind is `exportable: true`) — on scope-close, serialise to `<lib-dir>/<scope-kind>/<scope-name>.<artefact_kind>.facts.json`. No per-fact-kind export annotation needed.
6. **IMPORT** (parse-time, `@import_from_library { kind, name }` on rule) — lazy deserialise, merge into current scope; imported facts not re-exportable unless re-emitted.
7. **ROLLBACK** (parse-time, automatic, engine-internal) — `.3.3.3` IIFE transaction boundary; reads undo log; removes facts from every index they were inserted into; pops speculative scope opens; lazy-unloads speculative imports. O(operations-undone), never O(store).

**The protocol is exhaustive and closed:** every interaction with the store falls into exactly one of these seven stages. No new stages will ever be added. Adding a new fact-kind = ONE Stage 1 declaration; the other six stages are mechanical.

Worked example in §4.9 of the design proposal: `variable_binding` walked through all seven stages end-to-end. Same protocol applies to `macro_define`, `class_member`, `covergroup_bin`, `assertion_clock`, `constraint_membership`, or any kind we'll think of later.

**Why:** the parser cannot pre-decide which categories of facts will matter. Today's stress test is variable/type bindings for method-call disambiguation; tomorrow it could be macro definitions, virtual-method overrides, covergroup bins, constraint membership, assertion clocks — whatever future grammars need. A curated taxonomy in the engine becomes legacy debt the moment a new kind arrives. A schema-agnostic store with declaratively-indexed access is the parser-agnostic foundation.

**How to apply:**

- `@emit_fact { kind: K, name: N, <attrs>* }` is the universal producer. Engine treats `K` as an opaque label; never enumerates kinds.
- `@predicate has_fact / fact_attribute_equals / resolve_path / fact_count_at_least` are the composable query primitives. Grammar predicates compose them; engine never adds kind-specific predicates.
- Index schemas are declared per-fact-kind at grammar level (default: `(scope, kind, name)`; grammars trade memory for query speed by declaring extras like `(container)` for class members, `(position)` for diagnostics, `(name)` for dotted-path roots).
- Scope tree is real (not just a stack) so cross-scope queries (e.g., "members of class C from outside C") work; scope-kind labels (`file | package | class | function | block | …`) are opaque to engine, enforced by grammar discipline.
- Transactional rollback (the `.3.3.3` IIFE pattern, [[feedback_question_bypasses_manual_cleanup]]) extends across all indexes — speculative-parse retraction is O(emitted-in-tx), not O(store).

Captured in `docs/proposals/CONTEXT_AWARE_PARSING_DESIGN.md` §3 (Universal Store, Organisation Techniques, Scope Organisation, Query Layer, Performance + Scalability) and §6 (Engine Primitives — five real extensions identified, the first three of which are independent of method-call disambiguation and benefit every future grammar).

Reinforces [[feedback_ast_pipeline_parser_agnostic]] (every pipeline change must be a general primitive) and [[feedback_prefer_grammar_leave_engine_alone]] refined-form (engine changes ARE on the table when they're parser-agnostic features that benefit every parser).
