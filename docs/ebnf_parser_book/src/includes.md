# The Include System

> ⛔ **STATUS (measured 2026-07-26, `LANG-CAPABILITY-AUDIT.4`): the directives below are
> currently RECOGNIZED AND DISCARDED — they do not compose anything yet.**
>
> The shipping Rust frontend detects an include directive and skips the line
> (`rust/src/ebnf_frontend.rs:152-155`). No file is read and no rule is merged, **with exit
> code 0 and no diagnostic**. A grammar that relies on an include therefore loads with only
> its own rules, and the linter then reports the included rules as *undefined references
> (likely a typo)* — a misleading message that names the wrong cause.
>
> This is a regression, not a design: the resolution logic described in this chapter was
> implemented in the retired Perl frontend (`perl/AST/Transform.pm`) and was never carried
> over. **Restoring it is directed work** — see `docs/tasks/LANG-CAPABILITY-AUDIT.md` leaf
> `.7`, which also makes `--lint-grammar` honour the include graph. This notice is removed
> as part of that leaf.
>
> **Until then:** keep a grammar in a single file. Everything below describes the intended
> and once-working behaviour, and is the contract `.7` restores.

The include system lets you split a large grammar across multiple `.ebnf` files and share common rule
libraries. The directives are recognized by the EBNF frontend (`rust/src/ebnf_frontend.rs`) and — once
`.7` lands — resolved into a single combined grammar before code generation. The authoritative,
exhaustive reference — search paths, environment variables, recursive resolution, and cycle handling — is
`docs/EBNF_INCLUDE_SYSTEM.md`; this chapter is the author-facing summary.

## File includes

```ebnf
# short form (most common)
include("tokens", "expressions", "statements")

# full form
include_file("tokens.ebnf", "operators")

# alias
file("common")
```

- Includes the named files, in the order listed.
- The `.ebnf` extension is added automatically when you omit it — `include("common")` and
  `include("common.ebnf")` are equivalent.
- Multiple files may be named in one directive.

## Directory includes

```ebnf
# include ALL .ebnf files from these directories
include_dir("tokens", "expressions")

# alias
dir("../shared", "./components")
```

- Pulls in every `*.ebnf` file in each named directory.
- Files are processed in **alphabetical** order.

## How resolution works (summary)

1. The main grammar is scanned for include directives and rules.
2. Each directive is resolved to concrete file paths against the search path (the grammar's base
   directory, explicit `include_dir` directories, the `EBNF_INCLUDES` / `EBNFLIB` environment variables,
   then the current directory — see `docs/EBNF_INCLUDE_SYSTEM.md`).
3. Each included file is parsed and **recursively** processed for its own includes.
4. **Circular includes are handled gracefully** — each file is processed once; a cycle does not loop
   forever.
5. All rules from all files are combined into a single grammar; every rule name is in scope everywhere.

Because all included rules share one flat namespace, **avoid rule-name collisions** across files — two
files defining the same rule name is a well-formedness concern (`--lint-grammar`).

## A cautionary example: the meta-grammar's own include

`grammars/ebnf.ebnf:18` opens with:

```ebnf
include(semantic_annotations)
```

This was written as the meta-grammar composing in the semantic-annotation rules rather than restating
them. Measured, it does neither — **and it is a good illustration of why leaf `.7` exists**:

- the directive is discarded, so nothing is composed in; and
- **`grammars/semantic_annotations.ebnf` does not exist.** The nearest tracked files are
  `semantic_annotation.ebnf` and `builtin_semantic_annotation.ebnf`.

A dangling include pointing at a missing file survived in the meta-grammar precisely because the
directive is dropped before anything tries to resolve it. When `.7` lands, an unresolvable include
becomes a hard, named error, and this line must be fixed or removed.

## Organizing a multi-file grammar

A common functional split:

```text
grammars/foolang/
├── foolang.ebnf          # entry: include("tokens", "expressions", "statements")
├── tokens.ebnf           # keywords, identifiers, literals, trivia
├── expressions.ebnf      # operator precedence chain
└── statements.ebnf       # declarations, control flow, blocks
```

> Composition is done with **includes**, not the `import "…"` / `grammar … extends …` constructs the
> meta-grammar self-describes — those are not implemented and misparse (see
> [Rules and Expressions](rules-and-expressions.md)). Includes are the *designated* mechanism, and per
> the status notice at the top of this chapter they are the one being restored by
> `LANG-CAPABILITY-AUDIT.7`; the `import`/`extends` syntaxes are not.
