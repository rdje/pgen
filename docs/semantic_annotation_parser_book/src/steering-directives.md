# Steering Directives

This is the catalog of `@`-directives PGEN's AST pipeline **interprets**. Their *syntax* is the value
language ([Grammar and Scope](grammar-and-scope.md)); their *meaning* — how each one steers the
generated parser and the stimuli generator — is summarized here and governed in full by
`docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` and the steering control matrix
`docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` (the SC-01…SC-13 control slices, each a
Tier-4 gate-enforced contract).

Every example uses PGEN's real surface: the directive is written **on its own line above** the rule it
governs.

## Predicate gating (store-aware parsing)

A `@predicate` gates whether a rule (or branch) may match, by querying the **semantic store** (see
[The Semantic Store](semantic-store.md)). This is what makes a PGEN parser context-aware — e.g. "only
treat this identifier as a type name if a `type_name` fact for it was already emitted."

```ebnf
@predicate: has_fact(type_name, $head)
known_type_identifier := identifier
```

Built-in predicate functions (the store-query vocabulary):

| Function | True when… |
| --- | --- |
| `has_fact(kind, ref)` | a fact of `kind` exists for the resolved name |
| `lacks_fact(kind, ref)` | no fact of `kind` exists for the name |
| `fact_attribute_equals(kind, ref, attr, value)` | a `kind` fact exists whose `attr` equals `value` |
| `lacks_fact_attribute_equals(kind, ref, attr, value)` | no `kind` fact for the name has `attr == value` |
| `fact_count_at_least(kind, n)` | at least `n` facts of `kind` exist (up to this position) |
| `len_bounds(min, max)` | the matched text length is within `[min, max]` |
| `numeric_bounds(min, max)` | the matched numeric value is within `[min, max]` |
| `value_compare(lhs, op, rhs)` | the two resolved captures compare true under `op` (see below) |

**`value_compare` — a rule-span value comparison (not a store query).** Unlike the fact predicates
above, `value_compare` reads no store: it gates a rule on a comparison between **two of the rule's own
resolved captures**. `op` is a word form — `lt` · `le` · `gt` · `ge` · `eq` · `ne` — and both operands
are ordinary payload references (`$1`, `$name`, dotted/indexed):

```ebnf
@predicate: { name: value_compare, args: [$min, le, $max], phase: post }
counted_quantifier := "{" min:number "," max:number "}"
```

The comparison is value-oriented: numeric for every op when both operands parse as integers (`05`
equals `5`, `05 < 4` is false), with a deterministic lexical/textual fallback otherwise. It lets a
grammar own a cross-capture accept/reject rule (e.g. a `{5,4}`-is-out-of-order reject) declaratively.
This is a **rule-span** constraint — distinct from the **atom-scoped** value guards
`@range`/`@len`/`@enum`/`@regex` (and `len_bounds`/`numeric_bounds` above), which each constrain a
*single* matched value against a constant. A malformed shape is inapplicable (non-blocking); an
unresolvable reference fails the rule loudly.

Phasing and composition:

- `phase:` routes evaluation — `@predicate: {phase: branch, …}` makes the predicate **branch-local**
  (gates one OR alternative); the defaults are pre-rule / post-rule predicates.
- `@predicate_def:` defines a **named composed predicate** (a reusable boolean of the built-ins) that a
  later `@predicate` can invoke by name.

When a predicate rejects, the parser self-explains it in a trace as
`🚫 Rule '…' rejected by post predicate '…'` (see the platform book's debug toolbox).

## Fact and scope lifecycle

These directives write to the store. The lifecycle they drive is detailed in
[The Semantic Store](semantic-store.md).

| Directive | Purpose | Example |
| --- | --- | --- |
| `@emit_fact` | emit a fact into the store on a successful match | `@emit_fact: type_name` above a `typedef` rule |
| `@fact_kind:` | declare a fact kind and its attributes (e.g. `exportable: true`) | `@fact_kind: {kind: type_name, exportable: true}` |
| `@open_scope` / `@close_scope` | open / close a scope boundary | `@open_scope: {name: module}` above a module rule |
| `@export_to_library` / `@import_from_library` | export facts to / import facts from a cross-compilation-unit library artifact | `@export_to_library` above a package rule |

## Profiles

`@profiles` restricts a rule to one or more named grammar profiles; outside those profiles the rule
backtracks before normal parsing. This is how a single grammar serves multiple language editions (e.g.
SystemVerilog `sv_2017` vs `sv_2023`, or regex `pcre2` vs `relaxed`).

```ebnf
@profiles: ["sv_2023"]
sv_2023_only_construct := …
```

## Terminal transform

`@transform` attaches a canonical parse transform to a terminal, coercing the matched text to a typed
value. Only the canonical form steers code generation (SC-01):

```ebnf
@transform: str::parse::<i64>().unwrap_or(0)
integer := /[-+]?[0-9]+/
```

The canonical shape is `str::parse::<T>().unwrap_or(default)` (path-aware target type, e.g.
`std::primitive::i64`). Non-canonical transforms are validated but do not activate typed code paths.

## Generation steering — sample hints

These steer the stimuli generator's choice of literal for a rule. They do not affect parsing.

| Directive | Purpose |
| --- | --- |
| `@sample` / `@literal` / `@example` | explicit literal sample for the rule (bare quoted string ⇒ unquoted literal hint) |
| `@probe_sample` | probe-only hint — active **only** when the annotated rule is the generation entry, so target-driven replay can seed broad dependency rules without short-circuiting top-level generation |
| `@stimulus` | legacy named-routing alias; behaves like `@sample` |

## Branch selection and weighting

When a grammar is ambiguous, these make OR-branch selection (and weighted stimuli generation)
deterministic (SC-05/SC-06):

| Directive | Payload | Effect |
| --- | --- | --- |
| `@branch_policy` | `longest_match` (default) / `ordered` / `priority_first` | how OR branches are chosen |
| `@weight` | numeric | weighted-probability sampling bias for stimuli |
| `@priority` | numeric | deterministic tie-break (overrides `@precedence`) |
| `@precedence` | numeric | deterministic tie-break (lower than `@priority`) |
| `@associativity` | `left` / `right` | associativity tie-bias |

Conflict policy: `@priority` > `@precedence`; duplicate directives are last-wins.

## Error recovery and synchronization

Bound and steer recovery when all OR branches fail (SC-07):

| Directive | Purpose |
| --- | --- |
| `@recover` | enable/disable recovery for the rule |
| `@recover_budget` / `@recover_parse_budget` / `@recover_global_budget` | cap successful recoveries per-rule / per-parse / per-parser-lifetime (`0` disables) |
| `@sync` | configure a recovery marker token to scan to |
| `@panic_until` | recovery marker with precedence over `@sync` (or EOF fallback) |

## Value-domain constraints

Constrain the matched value's domain (SC-08); also drives in-domain stimuli synthesis:

| Directive | Purpose |
| --- | --- |
| `@range` | numeric range, e.g. `@range: "1..100"` |
| `@enum` | enumerated allowed values |
| `@regex` | regex pattern the value must satisfy |
| `@len` | length bounds on a string/array capture |

An unsatisfiable intersection of these emits `W_SEM_UNSATISFIABLE_VALUE_DOMAIN`.

## Cross-field / relational constraints

Relate captures to each other (SC-09):

| Directive | Purpose |
| --- | --- |
| `@constraint` | a relational expression over captures/references (positional `$1`, dotted `$a.b`, `.len`) |
| `@requires` | each listed reference must resolve to a non-empty capture |
| `@implies` | antecedent truth implies consequent truth |

## Token steering (regex atom matching)

Steer how a regex-atom rule matches/generates (SC-04), precedence `@pattern` > `@charset` >
`@token_class`:

| Directive | Purpose |
| --- | --- |
| `@token_class` | a known class family (`identifier`, `int`, `float`, `hex`, `whitespace`, …) |
| `@charset` | a character-class payload, e.g. `"A-Za-z_"` |
| `@pattern` | an explicit regex payload (highest precedence) |

A token-steering directive on a rule with no regex atom warns `W_SEM_TOKEN_STEERING_WITHOUT_REGEX_ATOM`.

## Coverage, negative-case, and determinism hints

| Directive | Slice | Purpose |
| --- | --- | --- |
| `@coverage_target` | SC-10 | mark a rule/branch as a coverage target (optional weight); emits `CoverageTargetEvent` |
| `@critical_path` | SC-10 | path-criticality; warns if no effective `@coverage_target` |
| `@invalid_case` | SC-11 | mark an expected-failure path; emits `NegativeCaseEvent` on failure |
| `@negative` | SC-11 | stimuli negative-case marker; warns if no `@invalid_case` |
| `@seed_group` | SC-12 | explicit group key for deterministic partition routing |
| `@deterministic_group` | SC-12 | enable stable behavior partitions; emits `DeterministicPartitionEvent` |

## Metadata (parsed + validated, no active steering)

`@type`, `@category`, `@kind`, `@effect`, `@deprecated`, `@description`, `@version`, `@since`,
`@author`, … are parsed and validated by the annotation validator but carry no runtime steering — they
are documentation/metadata. (These are the predefined names listed in
[Grammar and Scope](grammar-and-scope.md).)

## Unknown-directive policy

An unrecognized directive is handled by a policy of `ignore` / `warn` / `strict`; strict-mode warning
promotion is selector-controlled (`PGEN_STRICT_SEMANTIC_WARNING_CODES`). The bootstrap backend never
hard-fails on an unknown payload — it falls back to a `Raw` classification (see
[Backends](backends.md)).
