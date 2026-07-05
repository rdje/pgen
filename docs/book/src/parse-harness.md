# The Parse Harness — parsing an *arbitrary* grammar

> **Part II · Inside PGEN.** For contributors and advanced users. This chapter documents the
> **parse harness**: the capability to parse an arbitrary input string against an *arbitrary*
> grammar — one you have not registered, codegen'd, and compiled into the platform — using semantics
> identical to the shipped generated parsers. It is a structural component of the AST pipeline, in
> the same family as the [grammar linter](grammar-wellformedness.md), the
> [stimuli generator](stimuli-and-quality.md), and the parser generator
> ([Developer Architecture](developer-architecture.md)).

## The capability gap this closes

PGEN is, by design, a **codegen-to-Rust** platform: a grammar `grammars/foolang.ebnf` is compiled to
a Rust parser `generated/foolang_parser.rs`, wired into the parser registry, and compiled in. That is
exactly what makes the shipped parsers fast and trustworthy — and it means the only inputs the
toolbox can drive are the grammars that are *already* registered and built:

- `parseability_probe --parse <grammar> <file>` is **registry-keyed** — it dispatches on a
  compiled-in `grammar_name`.
- `ast_pipeline foo.ebnf --generate-parser` emits **Rust source** — running it needs a compile.
- The in-memory parsers in the tree are the EBNF *meta-grammar* parser (which parses `.ebnf` files,
  not arbitrary grammars' *inputs*) and the registered embedding-API grammars.

So a throwaway one-rule probe grammar like `r := "a" | "a" "b"` could not be parsed on the input
`"ab"` in one step to see *which alternative wins* — the precise `--parse-dump-ast-pretty` +
`--trace-rules` evidence style that decisively settles grammar-authoring and linter-soundness
questions (for example, "is this ordered-choice shadowing verdict sound under PGEN's *backtracking*
engine?"; see [Grammar Well-Formedness](grammar-wellformedness.md)). Nothing was *broken* — this was
a genuine, nameable **capability gap**. The parse harness closes it.

## What the harness delivers

Given **(a)** an arbitrary grammar (a `.ebnf` file or its normalized generation-input AST) and
**(b)** an input string, the harness produces the **parse verdict** (accept / reject, with
`furthest_position` on reject) and the **typed AST** — using semantics identical to the shipped
generated parsers — *without* the full production ceremony. Crucially, "identical semantics" is not a
hope: PGEN parsing is not purely structural. It runs return-annotation folding, `@predicate` store
gates, `@emit_fact`/the semantic store, the `branch_policy` that decides ordered-choice
(`longest_match` / `ordered` / `priority_first`), and lookahead `&`/`!`. A faithful harness must run
the **same runtime** the generated parser runs — which is the crux of *trust* (below).

## Three approaches, three trust profiles

The capability is delivered through three approaches, each with an explicit, re-checkable trust
basis. Two are **authoritative by construction** (they *are* the real pipeline, so they inherit its
correctness); one is **authoritative by verification** (a second implementation, made trustworthy by
a differential-equivalence oracle).

| Approach | Trust basis | Trusted surface | How it is checked |
|---|---|---|---|
| **Scratch-register slot** *(this chapter — landed)* | **by construction** — identical to the shipped register→codegen→drive pipeline | the small scratch registry wiring | an integration test: a known scratch grammar → known verdict/AST |
| **Compile-and-run harness** *(forthcoming — `PARSE-HARNESS.3`)* | **by construction** — runs the shipped codegen + runtime on a throwaway compile | the harness plumbing (codegen call, I/O marshalling) | an integration test reproducing a registered grammar's verdict/AST |
| **Grammar-AST interpreter** *(forthcoming — `PARSE-HARNESS.4`+)* | **by verification** — a shared-core dynamic dispatcher over the gen-AST | the thin dynamic-dispatch layer over the shared runtime | a differential-equivalence gate vs the generated parser, byte-for-byte, over the full corpus + a per-combinator suite |

The design in full — including how the interpreter is made "100 % trustworthy" via the
differential-equivalence oracle and the per-combinator suite — lives in the task tree
`docs/tasks/PARSE-HARNESS.md`. The rest of this chapter documents the **scratch-register slot**, which
is live today.

## The scratch-register slot

The scratch slot is a **blessed, throwaway parser** — `grammar_name` `scratch` — whose grammar body
is meant to be **overwritten freely**. Because it goes through the *real* register → codegen → drive
pipeline, whatever it reports is exactly what a first-class shipped parser would report. It is the
most **production-faithful** path and the one that yields the **richest evidence**: the entire
`parseability_probe` toolbox (`--parse`, `--parse-dump-ast-pretty`, `--trace-rules`,
`--report-certificate-coverage`) and `ast_pipeline --lint-grammar` all work on the synthetic grammar,
unchanged.

### The slot

- **`grammars/scratch/scratch.ebnf`** — the active probe grammar *and* the committed
  integration-test fixture. Its entry rule is named `scratch` so the registered slot name is stable;
  helper rules may be named anything.
- The generated artifacts (`generated/scratch_parser.rs`, `generated/scratch.json`) are **git-ignored**
  — they never enter the tracked set. `scratch.ebnf` **is** tracked.

### Workflow

```bash
# 1. Edit grammars/scratch/scratch.ebnf (keep the entry rule named `scratch`).
#    For example, to probe the fixed-prefix ordered-choice shape on "ab":
#        scratch := "a" | "a" "b"

# 2. Regenerate the scratch parser artifact and rebuild the probe:
make -C rust SHELL=/bin/bash focus_scratch
(cd rust && cargo build --release --features generated_parsers --bin parseability_probe)

# 3. Drive it with the full toolbox:
printf 'ab' > /tmp/in.txt
./rust/target/release/parseability_probe --parse scratch /tmp/in.txt
./rust/target/release/parseability_probe --parse-dump-ast-pretty scratch /tmp/in.txt /tmp/out.json
PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
    --parse scratch /tmp/in.txt --trace-rules scratch

# Static analysis + trustworthiness on the synthetic grammar:
./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf --lint-grammar
./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf \
    --report-certificate-coverage --entry-rule scratch --count 40 --seed 0
```

### How it works (the wiring)

The slot mirrors a shipped grammar (`json`) exactly, and adds nothing to the trusted surface beyond a
small, reviewable, one-time registration:

- **Makefile** (`rust/Makefile`) — `SCRATCH_EBNF` / `SCRATCH_JSON` / `SCRATCH_PARSER` variables and a
  `focus_scratch` target: `.ebnf → raw-AST JSON → parser.rs`, identical to `focus_json`.
- **`rust/build.rs`** — sets the `has_generated_scratch_parser` cfg (and the resolved `include!` path)
  **only when the artifact exists**. With no built artifact, the slot is simply *absent* — additive,
  never affecting any shipped grammar, and a clean checkout compiles unchanged.
- **`rust/src/lib.rs`** — `generated_parsers::scratch`, gated on that cfg.
- **`rust/src/parser_registry.rs`** — a `GeneratedParserRegistryEntry { grammar_name: "scratch", … }`
  plus the dispatch closures for parse, certificate-coverage witness, and labelled detail.

The one design subtlety worth calling out: a shipped grammar's registry closure hard-codes its entry
method (e.g. `parser.parse_full_json()`), but a *scratch* grammar's entry-rule name changes every time
you overwrite the body. The slot sidesteps this by dispatching through the generated parser's
**entry-rule-agnostic** `parse_full()` (the method every generated parser exposes; `parse_full_json`
is just a thin alias for it). So the registration is **stable across arbitrary probe grammars** — the
registry never needs to know the probe grammar's entry-rule name. An explicit
`--entry-rule RULE` still routes through `parse_full_from(RULE)`, so you can inspect a non-entry rule
(for example a shadowed alternative) in isolation.

### Why this is trustworthy — "by construction"

The scratch slot **is** the shipped parser generator plus the shipped runtime, driven exactly the way
every registered grammar is driven. There is no second parsing implementation to keep faithful, so
there is no semantic drift to worry about: return-annotation folding, `@predicate` gates, the semantic
store, `branch_policy`, lookahead, the quantifier engine, memoization, and the `furthest_position`
accounting are all the *real* ones. The only thing that could be wrong is the thin registration
plumbing, and that is pinned by an integration test
(`parser_registry.rs::scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast`,
gated on `has_generated_scratch_parser`): the committed default fixture must parse its known inputs to
the known verdict and AST. Restore the default fixture (`git checkout grammars/scratch/scratch.ebnf`)
before committing if you edited it for a probe, so that oracle keeps passing.

## Honest bounds

- The scratch slot and the compile-and-run harness are authoritative *by construction* — they run the
  real pipeline, so their trust burden is only the plumbing. They cost a per-probe codegen + compile
  (seconds), which is the price of "use the real thing."
- The forthcoming interpreter (`PARSE-HARNESS.4`+) is a genuine second implementation; its trust is
  *earned* by a differential-equivalence oracle, and the honest claim there will be "divergence-free
  over a combinator-complete, real-world, fuzzed corpus with a shared core" — **not** a formal
  all-inputs proof. See `docs/tasks/PARSE-HARNESS.md` §3.4.

## See also

- [Grammar Well-Formedness & Well-Definedness](grammar-wellformedness.md) — the linter-soundness
  questions the harness exists to answer empirically (e.g. ordered-choice shadowing under backtracking).
- [Diagnostic & Debug Toolbox](diagnosing-unknowns.md) and
  [Debugging With `parseability_probe`](parseability-probe-debug.md) — the toolbox the scratch slot
  makes available on synthetic grammars.
- `docs/tasks/PARSE-HARNESS.md` — the full design, trust architecture, and phasing for all three
  approaches.
