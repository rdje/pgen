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
| **3 — QUERY** | rule pre / branch / post (parse-time) + whole-input `final` (parse-completion) | `@predicate <name>(...)` | Reads from the store to gate or steer parsing; `final` defers a whole-input check (forward references) to parse completion. |
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

### Rule-level vs branch-local placement

`@emit_fact:` is most often attached at the **rule level** (on its own line above
`name :=`), where it fires when the whole rule commits. But a binding is
sometimes recognized in just **one branch** of an ordered choice — for example
only `import pkg::*;` opens a wildcard import, not `import pkg::name;`. For those
cases you can attach the action directive **inside the branch**, immediately
before that branch's items, exactly where the meta-grammar allows any inline
annotation:

```ebnf
package_import_item := package_identifier scope_resolution identifier
                          -> { kind: "explicit", package: $1, name: $3 }
                     | @emit_fact: { kind: wildcard_import_open, name: $package.body }
                       package_identifier scope_resolution star
                          -> { kind: "wildcard", package: $1 }
```

A branch-local action fires **only for the branch the parser selects**, resolved
against that branch's captured content, and rides the same speculation-safe
rollback as a rule-level emit: a branch that is tried and then loses the ordered
choice never leaks its emission, and the whole emission is undone if the
enclosing rule later backtracks (Stage 7). This is the **action** counterpart to
a branch-local `@predicate … phase: branch` (which *steers* the choice): the same
inline placement carries `@emit_fact`, `@open_scope`, and `@close_scope` too.

Prefer rule-level placement for a single-branch rule (it is equivalent and
clearer); reserve branch-local placement for emitting from one specific branch of
several.

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

**Name matching is textual.** A query's name argument matches a stored fact
(or scope) name by its **text**, not by how either side was written: a quoted
`"special"`, an unquoted `special`, and a `$ref` that resolved to `special`
all denote the same name (case-sensitively). Kinds and attribute keys match
case-insensitively; attribute values match textually too. *History:* until
FACT-NAME-MATCHING.2 (2026-07-06) name matching was variant-strict — a quoted
`"x"` could never match a `$ref`-emitted name, a silently dead gate — which is
why older grammars follow an unquoted-identifier convention in `args:`. Both
forms are now equivalent.

### Phases

The `phase:` modifier tells the engine *when* to check:

- `pre` — before the rule's body parses. Failure rejects the rule
  immediately (saves work).
- `branch` — inside an ordered choice, gating which branch fires.
- `post` — after the rule's body parses, before committing the
  transaction. Failure backs out the rule plus its fact emissions.
- `final` — **not** an inline gate on its own rule: a whole-input assertion
  checked **once, after the top-level parse succeeds and consumes the full
  input**, against the now-complete store. Use it to validate a **legal forward
  reference** — a reference whose definition may appear *later* in the input
  than the reference itself (see below).

#### `phase: final` — whole-input / forward-reference checks

`pre`, `branch`, and `post` all fire at the reference's *own* parse position, so
they can only see facts emitted *so far*. That is fine for declare-before-use,
but it cannot express **use-before-declare**: a `post has_fact` gate on a
reference fires *before* a later definition is emitted and would wrongly reject
a legal forward reference. `phase: final` closes this gap.

A `final` predicate resolves its args against the carrying rule's captured
content **at rule commit** (so `$name` becomes the concrete captured string),
enqueues a **deferred obligation**, and is discharged **once** at whole-input
parse completion against the complete store. It rides speculation rollback
exactly like `@emit_fact` — an obligation registered on a losing/failed branch
never survives — and obligations discharge in ascending source-position order,
so the first that fails reports a validator-style first-error-by-position.
It composes with the entire predicate vocabulary (`has_fact`,
`fact_count_at_least`, …): `final` is the *when*, not the *what* — the
whole-input generalization of `post`.

```ebnf
# Accept `\k<aa>(?'aa'x)` (forward reference), reject a name defined nowhere.
@emit_fact: { kind: capture_name, name: $name }              # a definition, anywhere
named_group := "(?'" capture_name "'" body ")"

@predicate: { name: has_fact, args: [capture_name, $name], phase: final }
backreference := "\\k<" capture_name ">"  -> { ref: $2 }     # a use — order-independent
```

**Named-reference resolution falls back across views (`view: shaped` is now
optional self-documentation).** A `@predicate` defaults to `view: raw` and a
`final` predicate resolves its args at rule commit. A **named / object-key**
reference (`$name`, `$ref`, dotted `$a.b`) that is absent from the view-selected
content **falls back to the other content view** before raising the
unresolved-attribute error (`FINAL-PHASE-PREDICATE.3`). So a shaped-key reference
resolves under the default `view: raw` on **any** rule shape — a single-branch
rule (whose raw capture coincides with the shaped `-> {…}` object) *and* a
**multi-branch** tournament (whose winning branch captures a raw `Sequence` that
holds no `ref` element — the reference then resolves against the shaped view via
the fallback). The fallback fires **only** on a named-reference miss; positional
`$N` references walk the raw tree structurally and are never retried against the
other view, and a key absent from **both** views still errors (a real typo is never
masked). Writing `view: shaped` explicitly is therefore optional — it documents
intent and pins resolution to the produced object, but is no longer required for
correctness. The three regex named-reference gates (`named_backreference`,
`named_subroutine_target`, `python_named_backreference`,
`REGEX-PCRE2-FIDELITY.4.11`) keep their explicit `view: shaped` as
self-documentation.

**Grammar-author rule of thumb.** For "reference X must resolve against a
definition that may appear anywhere, including later," use `phase: final` — not
`post` (which rejects the legal forward reference) and not an out-of-band host
validator (invisible to the stimuli generator, against the
EBNF-single-source-of-truth doctrine). Reserve a real two-pass pre-scan only for
an *irreversible pre-emission global aggregate*; a pure reference validator has
no such dependency, so it defers. `final` predicates are generation-neutral (they
gate no branch during the pass), so a consumer keeps its existing conservative
`@gen_predicate` draw — the generator only ever references already-emitted
names, so it never produces an invalid forward reference.

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

### Value-comparison predicates: `value_compare`

Not every predicate reads the store. `value_compare` is a built-in that gates a
rule on a **comparison between two of the rule's own resolved captures** — a
*rule-span value constraint*:

```ebnf
# Reject when the counted-quantifier minimum exceeds the maximum:
@predicate: { name: value_compare, args: [$min, le, $max], phase: post }
counted_quantifier := "{" min:number "," max:number "}"
```

- `<op>` is one of `lt` · `le` · `gt` · `ge` · `eq` · `ne` (word forms — the op
  is a plain identifier argument, not a symbol).
- `$lhs` / `$rhs` are any resolvable payload reference (`$1`, `$name`, dotted
  `$a.b`, indexed `$a[0]`), resolved against the rule's captured content just
  like every other directive-payload reference.
- The comparison is **value-oriented**: when both operands parse as integers the
  comparison is numeric for *all* ops — so `05` equals `5` and `05 < 4` is false,
  regardless of leading zeros — with a deterministic lexical/textual fallback for
  non-numeric operands. It holds iff `resolve($lhs) <op> resolve($rhs)` is true;
  a `post`-phase failure rejects the rule (and backs out its emissions).

This is the primitive that lets a grammar own an accept/reject rule that only
compile-time Rust used to express (the motivating case is PCRE2's
`{5,4}`-is-out-of-order rule). It is deliberately distinct from two neighbours:

- it is **not** a store query — it never touches facts, so it needs no
  `@fact_kind`/`@emit_fact` and rolls back nothing store-related;
- it is a **rule-span** constraint (it compares two *different* captures),
  categorically distinct from the **atom-scoped** value guards
  (`@range`/`@len`/`@enum`/`@regex`), each of which constrains a *single* atom's
  matched text against a constant.

Like every built-in predicate, a malformed shape (wrong arity, or an unknown op
word) evaluates to *inapplicable* (non-blocking); an **unresolvable** `$ref`
argument is a loud grammar-author error (the rule fails), never a silent pass.

#### Comparing by code point: `value_compare_codepoint`

`value_compare` compares operands as **values** (integer when both parse, textual
otherwise). Sometimes a rule needs to compare two captures by their decoded
**Unicode code point** instead — for example a character-class range endpoint,
where the two ends may be written as bare characters *or* escape spellings and
the constraint is on their code points, not their text. `value_compare_codepoint`
is the sibling built-in for exactly that:

```ebnf
# Reject a descending class range ([z-a], [\x{100}-a]): endpoints compared by code point.
@predicate: { name: value_compare_codepoint, args: [$1, le, $5], phase: post }
class_range := class_atom "-" class_atom -> { start: $1, end: $5 }
```

It takes the same op words (`lt`·`le`·`gt`·`ge`·`eq`·`ne`) and the same payload
references. The difference is coercion: each operand is decoded as a single
**character literal** — a bare Unicode scalar, or the standard C/Perl
character-escape vocabulary shared across languages (hex `\xHH` / `\x{H..}`,
octal `\o{O..}` / `\NNN`, control `\cX`, the named escapes `\a \b \e \f \n \r
\t`, or a backslash-escaped literal `\X`) — to its code point, and the two code
points are compared numerically. This is essential where a textual comparison
misleads: `"\x{100}"` sorts textually *before* `"\x{FF}"` (because `'1' < 'F'`),
but the code points are `256 > 255`. An operand that is not a single decodable
character literal makes the comparison *inapplicable* (non-blocking), the same
convention `value_compare` follows. It decodes character literals, not grammar
constructs, so it is fully parser-agnostic.

### Scope-context predicates: `in_scope_kind` / `not_in_scope_kind`

Some rules are legal only *inside* — or only *outside* — an enclosing
construct, at any nesting depth. `in_scope_kind` and its complement
`not_in_scope_kind` are the built-ins for that **lexical-containment** question.
Each takes a single scope-*kind* argument and asks whether the parser is
currently inside an open scope of that kind:

```ebnf
# Reject \K anywhere inside a lookaround body (PCRE2 err 199), but accept it
# elsewhere: the lookaround open-marker opens a `lookaround` scope, and the
# keep-out anchor only parses when NOT inside one.
@predicate: { name: not_in_scope_kind, args: [lookaround], phase: pre }
keep_out_anchor := "\\K" -> {type: "anchor", kind: "keep_out"}
```

- `in_scope_kind(kind)` holds iff **any** currently-open scope — the innermost
  frame *or any enclosing ancestor up to and including the global scope* — has
  that kind. `not_in_scope_kind(kind)` is its boolean complement (the
  `has_fact`/`lacks_fact` pairing), so "reject X inside Y" needs no
  `@predicate_def` negation wrapper.
- It is the **whole-active-chain** generalization of the innermost-only
  `current_scope_is`: where `current_scope_is(lookaround)` sees only the deepest
  frame (and so *misses* a `\K` nested inside a plain group inside the
  lookaround — `(?=a(b\Kc))`), `in_scope_kind(lookaround)` walks the entire
  active chain and still sees the enclosing lookaround.
- It reads the **live** scope chain, so it **auto-unwinds** the instant an
  `@close_scope` pops the scope — a `\K` written *after* the lookaround
  (`(?=ab)\K`) is allowed again. This is the property the store's
  facts cannot provide: emitted facts are global and never retracted on
  `@close_scope`, so a leak-free containment gate has to read scopes, not facts.
- The argument is a scope *kind* only (a built-in kind such as `function` /
  `block`, or any custom label like `lookaround`); a name-filtered variant is a
  possible future extension. A missing/empty kind evaluates to *inapplicable*
  (non-blocking), the same convention every built-in follows. It reads only the
  scope chain, so it is fully parser-agnostic.

This is **live in the shipped regex grammar** as of parser release `1.1.101`
(`REGEX-PCRE2-FIDELITY.4.10`): each of the seven lookaround rules opens a
`lookaround` scope at its opener (via a small open-marker rule, since
`@open_scope` fires *after* the rule body), and the `keep_out` rule carries
`@predicate not_in_scope_kind(lookaround)` — the first grammar consumer of this
primitive. It replaced an out-of-band host validator check (single source of
truth), byte-neutral at the released parse surface.

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

**The boundary is transactional, not structural.** Effects commit on rule
**success** — *including a zero-length success*. A quantifier's zero-length
guard discards a zero-length iteration *structurally* (it contributes no
node, preventing an infinite loop), but that guard is not a transaction:
the discarded iteration's rule *succeeded*, so its `@emit_fact` persists
(normative — the *Effects Timing* section of
`PGEN_ANNOTATION_NORMATIVE_SPEC.md`, pinned differentially by the
`sem_zero_len_emit` suite case). A zero-width marker emission is a
legitimate idiom; if you want no-effect-when-empty, make the emitting rule
consume at least one byte.

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

# Query (any phase: pre / branch / post / final):
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
