# The Semantic Store

The store is the parser's memory: a set of **facts** organized into **scopes**, written and queried by
the steering directives, and **transactional** so that backtracking never leaves stale state. This
chapter summarizes the lifecycle the directives drive; the platform book's *The Semantic Store: Parser
Memory* chapter is the full treatment.

## The seven-stage lifecycle

| Stage | Driven by | What happens |
| --- | --- | --- |
| **DECLARE** | `@fact_kind:` | declare a fact kind and its attribute schema (e.g. `exportable: true`) |
| **EMIT** | `@emit_fact` | on a successful match, write a fact (a name + attributes) into the current scope |
| **QUERY** | `@predicate` built-ins (`has_fact`, `fact_attribute_equals`, `lacks_fact`, …) | read the store to gate a rule/branch |
| **SCOPE** | `@open_scope` / `@close_scope` | push/pop scope boundaries; queries resolve along the scope path |
| **EXPORT** | `@export_to_library` | publish exportable facts to a cross-compilation-unit library artifact |
| **IMPORT** | `@import_from_library` | pull facts from a previously-exported library artifact (e.g. an imported package) |
| **ROLLBACK** | (automatic, transactional) | when a speculative parse backtracks, the facts it emitted are rolled back |

## Worked shape

A typical declare → emit → query loop (SystemVerilog-style typedef visibility):

```ebnf
# DECLARE the fact kind once (often near the top of the grammar)
@fact_kind: {kind: type_name, exportable: true}

# EMIT a type_name fact when a typedef is parsed
@emit_fact: type_name
type_declaration := 'typedef' data_type identifier ';'

# QUERY it: only accept a bare identifier as a type name if it was declared
@predicate: has_fact(type_name, $head)
known_type_identifier := identifier
```

Here the parser will only accept `my_t x;` as a typed declaration *after* it has seen
`typedef … my_t;` — the store carries that fact forward. This is the mechanism behind PGEN's
context-aware grammars; the platform book and the SystemVerilog parser book have deeper worked
examples.

## Scopes

`@open_scope` / `@close_scope` build a scope tree so that a fact emitted inside a module/package/class
is visible to queries within that scope and resolved along the scope path. This is what lets the same
name mean different things in different contexts.

## Cross-unit visibility: export / import

`@export_to_library` writes the **exportable** facts (those whose `@fact_kind:` declared
`exportable: true`) to a library artifact; `@import_from_library` reads them back in another
compilation unit. This is how, for example, an `import pkg::*;` can make a package's exported type
names visible in the importing file.

## Transactions and rollback

The store is transactional. PEG parsing speculatively tries alternatives and **backtracks**; when it
does, any facts emitted on the abandoned path are rolled back, and — critically — PGEN's packrat memo
replays the *semantic delta* on a cache hit, so a memoized result re-applies its fact emissions rather
than silently dropping them. The net effect: a fact is in the store **iff** the committed parse that
emitted it survived. This soundness is what makes store-aware gating trustworthy; it is also what the
store-aware generation work relies on so the stimuli generator never emits a sample its own parser
would semantically reject.

## Observability

When a predicate query rejects a rule, the debug toolbox makes the *why* explicit — a scoped
`--trace-rules <rule>` at `high`/`debug` prints the predicate verdict and the resolved facts (see the
platform book's *Diagnostic & Debug Toolbox* chapter and the 3-step `UNKNOWN` protocol).
