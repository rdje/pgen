# The Include System

The include system lets you split a large grammar across multiple `.ebnf` files and share common rule
libraries. The directives are recognized by the EBNF frontend (`rust/src/ebnf_frontend.rs`) and resolved
into a single combined grammar before code generation. The authoritative, exhaustive reference — search
paths, environment variables, recursive resolution, and cycle handling — is `docs/EBNF_INCLUDE_SYSTEM.md`;
this chapter is the author-facing summary.

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

## A real example: the meta-grammar includes itself's annotation support

`grammars/ebnf.ebnf` opens with:

```ebnf
include(semantic_annotations)
```

That is how the EBNF meta-grammar composes in the semantic-annotation rules rather than restating them —
the same mechanism you use to factor your own grammar into `tokens`, `expressions`, and `statements`
files.

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
> meta-grammar self-describes — those are not implemented (see
> [Rules and Expressions](rules-and-expressions.md)). The include system is the supported mechanism.
