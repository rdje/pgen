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
| **Scratch-register slot** *(landed — `PARSE-HARNESS.2`)* | **by construction** — identical to the shipped register→codegen→drive pipeline | the small scratch registry wiring | an integration test: a known scratch grammar → known verdict/AST |
| **Compile-and-run harness** *(landed — `PARSE-HARNESS.3`)* | **by construction** — runs the shipped codegen + runtime on a throwaway *external* compile | the harness plumbing (codegen call, throwaway-crate synthesis, I/O marshalling) | an integration test reproducing a registered grammar's verdict + byte-identical AST |
| **Grammar-AST interpreter** *(forthcoming — `PARSE-HARNESS.4`+)* | **by verification** — a shared-core dynamic dispatcher over the gen-AST | the thin dynamic-dispatch layer over the shared runtime | a differential-equivalence gate vs the generated parser, byte-for-byte, over the full corpus + a per-combinator suite |

The design in full — including how the interpreter is made "100 % trustworthy" via the
differential-equivalence oracle and the per-combinator suite — lives in the task tree
`docs/tasks/PARSE-HARNESS.md`. The two **by-construction** approaches are live today; this chapter
documents both.

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

## The compile-and-run harness

The compile-and-run harness is the second **by-construction** path. Where the scratch slot makes a
probe grammar a *registered, compiled-in* parser (and so needs a `pgen` rebuild and a
`parseability_probe` rebuild to pick up a body change), the compile-and-run harness is
**self-contained**: it takes an arbitrary `.ebnf` and an input, runs the real codegen, compiles the
emitted parser as a **throwaway crate**, runs it, and hands back a structured result — touching neither
the registry nor `pgen`. That self-containedness is exactly why it is the natural **CI oracle** for the
grammar-AST interpreter's differential-equivalence gate (`PARSE-HARNESS.5`): a gate can call it in-process
on any grammar without editing the platform.

It is exposed as a small, parser-agnostic Rust API — `pgen::parse_harness`:

```rust
use pgen::parse_harness::{compile_and_parse, CompileAndParseOptions, ParseOutcome};
use std::path::Path;

let outcome: ParseOutcome = compile_and_parse(
    Path::new("grammars/scratch/scratch.ebnf"),
    "hello, world!",
    &CompileAndParseOptions::default(),
)?;
assert!(outcome.accepted);          // full-parse verdict
// outcome.furthest_position        — deepest byte reached (the reject locus on !accepted)
// outcome.ast_json: Option<Value>  — the typed AST on accept (byte-identical to the registry's)
// outcome.error:    Option<String> — the reject message on !accepted
```

`CompileAndParseOptions` lets you parse from an alternate entry rule (`entry_rule`, routed through
`parse_full_from`), point at a specific `ast_pipeline` codegen binary or `pgen` crate dir, and reuse a
working directory across probes so the `pgen` dependency stays compiled (see *Cost & reuse* below). A
parse **rejection** is a successful `Ok(ParseOutcome { accepted: false, .. })`, not an error; the
`HarnessError` type is reserved for genuine *plumbing* failures (missing codegen binary, a
codegen/compile/run failure, unparseable probe output), each carrying the failing step + captured tool
output so it is actionable.

### How it works (the plumbing)

The harness adds nothing to the trusted surface beyond this five-step plumbing:

1. **Codegen — the real thing.** It shells the shipped `ast_pipeline` binary once, reading the `.ebnf`
   directly (`ast_pipeline foo.ebnf --generate-parser --eliminate-left-recursion -o …`). This is the
   shipped `RUST_GENERATOR` recipe minus the `--debug`/`--trace` *logging* flags — verified to produce
   **byte-identical** parser source (the only difference from `make focus_<grammar>` is the embedded
   diagnostic output-path label, which never appears in the typed AST). The binary must carry
   `--features ebnf_dual_run` to read a `.ebnf` directly; the standard tree's `target/debug/ast_pipeline`
   does.
2. **Discover the struct name.** It greps the emitted source for its single
   `pub struct <Name>Parser<'input>` (exactly one per generated file — verified across every shipped
   grammar), so it never has to re-derive the codegen's grammar-name → struct-name mapping.
3. **Synthesize a throwaway crate.** A tiny `Cargo.toml` (path-dependency on `pgen`; deps `rustc-hash`,
   `regex`, `serde_json`) and a generated `main.rs` that `include!`s the emitted parser, constructs
   `<Name>Parser::new(input, NoOpLogger)`, calls `parse_full()` / `parse_full_from(entry)`, and prints a
   sentinel-fenced JSON result.
4. **Compile it — isolated.** `cargo build` runs with an isolated `CARGO_TARGET_DIR`, so the nested
   build never contends on an outer `cargo test`'s `pgen` target lock.
5. **Marshal the result back.** The harness runs the probe binary on the input (passed via a file, so
   arbitrary bytes survive), reads the JSON between the sentinels, and returns the `ParseOutcome`.

### The load-bearing fact: a generated parser compiles *outside* `pgen`

The design hinges on one non-obvious question. A generated parser's source hard-codes
`use crate::ast_pipeline::{…}`. *Inside* `pgen` that resolves against the whole crate — including
`pub(crate)` items an external crate cannot see — so it was not obvious a generated parser could compile
in a *throwaway external* crate at all. A static audit settles it: every `generated/*_parser.rs` reaches
**only** `crate::ast_pipeline::*` (22 distinct symbols, **all `pub`**) plus the externs `regex`,
`rustc_hash`, `serde_json`. So the throwaway needs just one shim — `use pgen::ast_pipeline;` at its crate
root, which makes the generated file's `crate::ast_pipeline::…` paths resolve to `pgen`'s module — and it
compiles. And because `ParseNode` derives `serde::Serialize`, the throwaway serializes the **same** typed
AST the registry's `parse_node_to_json` (`serde_json::to_value(node)`) does — so the harness's AST is
byte-identical to the shipped parser's, which the integration test asserts directly against the `json`
registry.

### Why this is trustworthy — "by construction"

Like the scratch slot, the compile-and-run harness **is** the shipped parser generator plus the shipped
runtime — it just compiles the result in a throwaway crate instead of the platform. There is no second
parsing implementation, so no semantic drift: the runtime that runs is the *real* one. The only thing
that could be wrong is the plumbing, and that is pinned by an integration test
(`parse_harness.rs::compile_and_run_harness_reproduces_json_registry_verdict_and_ast`, gated on
`has_generated_json_parser`): the harness must reproduce the `json` registry's accept/reject verdict and
its **byte-identical** typed AST on the same inputs.

### Cost & reuse

The dominant cost is the throwaway crate's **first** `cargo build`, which compiles the `pgen` lib as a
dependency (cold: seconds→a couple of minutes). Reuse the same `CompileAndParseOptions::workdir` across
probes and only the tiny probe bin recompiles — `pgen` stays cached in the isolated target dir — so a
batch driver (the `PARSE-HARNESS.5` equivalence gate) pays the `pgen` compile once.

## Honest bounds

- The scratch slot and the compile-and-run harness are authoritative *by construction* — they run the
  real pipeline, so their trust burden is only the plumbing. They cost a per-probe codegen + compile
  (seconds; the compile-and-run harness's *first* probe also compiles `pgen` as a dependency), which is
  the price of "use the real thing."
- The compile-and-run harness's by-construction guarantee is proven on `scratch` and `json` (which
  together exercise ordered choice, quantifiers, regex tokens, return-annotation folding, and
  memoization). A very large grammar such as full SystemVerilog could in principle emit a call to a
  `pub(crate)` runtime *method* that an external crate cannot see; if so, exposing that one method is a
  small, bounded follow-up (surfaced by the `PARSE-HARNESS.5` gate), not a redesign.
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
