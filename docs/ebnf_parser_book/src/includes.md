# The Include System

The include system lets you split a large grammar across multiple `.ebnf` files and share common rule
libraries. The directives are resolved by the EBNF frontend (`rust/src/ebnf_frontend.rs`) into a single
combined grammar before anything downstream — codegen, `--lint-grammar`, stimuli generation, the parse
harness — sees it. Every EBNF consumer in the repository funnels through that one entry point, so
include support is universal rather than per-tool. The authoritative, exhaustive reference — search
paths, environment variables, recursive resolution, and cycle handling — is
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
2. Each directive is resolved to concrete file paths against the search path, in this order:
   the **including file's own directory**, the directories of any files that included it, the
   `EBNF_INCLUDES` and `EBNFLIB` environment variables (colon-separated), then the current
   directory. The first match wins, so the nearest definition takes precedence.
3. Each included file is parsed and **recursively** processed for its own includes. An included
   file's own directory goes on the search path for *its* includes, so a subtree can use paths
   relative to itself.
4. **Circular includes are handled gracefully** — each file is composed in exactly once, so a
   cycle terminates and a diamond (two files including a third) does not duplicate rules.
5. All rules from all files are combined into a single grammar; every rule name is in scope
   everywhere.

**Rule order:** the main grammar's own rules come **first**, then the included rules in directive
order. This matters beyond tidiness — reachability analysis and the linter treat the first rule as
the grammar's canonical entry point, so an included file can never silently re-root your grammar.

> ⛔ **An include that cannot be resolved is a hard error**, naming the spec, the directive and the
> search path that was tried. It is never skipped. Before this was enforced, a mistyped or missing
> include vanished silently and the linter then blamed the *referencing rule* for an "undefined
> reference (likely a typo)" — pointing at everything except the actual cause.

### Rule names must be unique across the composed grammar

All included rules share one flat namespace, and **a rule name may be defined by only one file**.
Two different files defining the same name is a **hard error at load**, naming the rule and both
files:

```
duplicate rule definition: 'value' is defined in BOTH 'main.ebnf' and 'shared.ebnf'.
A rule reference must resolve to exactly one definition, so the same rule name may not be
defined by two different files. Rename one, or remove the duplicate include.
```

This is the invariant the whole system rests on: **any rule reference resolves to one and only
one rule definition.** Before it was enforced, a collision merged silently — one file quietly
added alternatives to another file's rule, changing what that rule accepted with no diagnostic.

> **Not affected:** repeating a rule header **within a single file**. Those clauses merge into
> alternatives of one rule and are an established idiom — see
> [Multi-clause definition](rules-and-expressions.md#multi-clause-definition-repeating-the-header-instead-of-chaining-).
> The unit of uniqueness is the *file*, not the clause.

There is deliberately **no override mechanism** — you cannot include a base grammar and redefine
one of its rules. If you need a variant, give it a different name.

## A real example: the SystemVerilog profiled wrapper

`grammars/systemverilog_lrm_profiled_wrapper.ebnf` is a 32-line file whose body is one directive:

```ebnf
include("systemverilog_lrm_profiled_generated")
```

It composes in the 1,400-rule generated grammar and adds its own wrapper rules on top — the whole
point of a wrapper grammar, and exactly the "factor the big part out" shape this system exists for.

> **A cautionary note from the same repair.** Until includes were resolved for real, that wrapper
> loaded **3 of its 1,400 rules** and reported success, and `grammars/ebnf.ebnf` carried an
> `include(semantic_annotations)` naming a file that has never existed. Neither could be detected,
> because the directive was discarded before anything tried to resolve it. Both are fixed: the
> wrapper composes, and the meta-grammar's stale directive was removed (it lints
> `undefined_references=0` without it — it was always self-contained). That is why an unresolvable
> include is now a hard error rather than a skipped line.

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
> [Rules and Expressions](rules-and-expressions.md)). The include system is the supported mechanism.
