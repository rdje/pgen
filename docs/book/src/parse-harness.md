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
| **Grammar-AST interpreter** *(landed — `PARSE-HARNESS.4` core + `.5` gate)* | **by verification** — a shared-core dynamic dispatcher over the gen-AST | the thin dynamic-dispatch layer over the shared runtime | the **differential-equivalence gate** vs the generated parser, byte-for-byte over a deterministic corpus (`PARSE-HARNESS.5`, below) |

The design in full — including how the interpreter is made "100 % trustworthy" via the
differential-equivalence oracle and the per-combinator suite — lives in the task tree
`docs/tasks/PARSE-HARNESS.md`. All three approaches are live; this chapter documents each. The
interpreter's `.4` **core** is byte-identical to the generated parser on the structural +
return-annotation surface, and the `.5` **differential-equivalence gate** now certifies **all 11 registered
grammars byte-identical** (including SystemVerilog / VHDL / rtl_frontend, and — since `.5.1`–`.5.5` —
`regex`, `systemverilog_preprocessor`, `ebnf`, `return_annotation`, and `rtl_const_expr`); the DEFERRED
ratchet is now empty (every previously-deferred grammar has been promoted — see *The differential-equivalence
gate* below). The remaining work is the semantic-directive orchestration (`PARSE-HARNESS.6`).

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

### The canonical probe: which alternative wins? (TOOLBOX Protocol D)

The slot's signature use is the question that motivated the whole harness: *which alternative does
the engine actually select — is this branch live or dead?* Never answer it from the grammar text
(PGEN's `|` is a branch **tournament**, `longest_match` by default — *not* PEG first-match commit).
Isolate the choice in the slot (carrying the real rule's `@branch_policy`/`@priority` annotations),
then read the winner off codegen's own selection line:

```bash
PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
    --parse scratch /tmp/in.txt --trace-rules scratch 2>&1 | grep 🏁
# → 🏁 Rule 'scratch' selected branch 2/2 consuming 2 chars (priority=0, associativity=left, branch_policy=longest_match)
```

That one line names the winning branch, the policy that chose it, and the consumed length; the AST
dump confirms the winner's shape. This is exactly how `GRAMMAR-WELLFORMED.A2.3` was decided (the
harness's first real use): the engine selected the very alternative the linter's old unconditional
fixed-terminal-prefix verdict branded unreachable, and the verdict was made branch-policy-aware as a
result. The full step-by-step lives in `TOOLBOX.md` **Protocol D**.

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

## The grammar-AST interpreter

The interpreter is the third approach — and the only one that is **not** authoritative by construction.
Where the scratch slot and the compile-and-run harness *are* the shipped pipeline (they run the real
codegen + runtime), the interpreter is a genuine **second parsing implementation**: it parses an input
against an arbitrary grammar **in-process, with no codegen and no compile**, by *dynamically
dispatching* over the normalized generation-input AST (the same `ASTNode` IR `--dump-gen-ast` emits and
that codegen consumes) instead of running generated match-arms. It is the fast, in-process capability
the director named — the tool for a daily grammar-authoring probe or a linter-soundness question, once
trusted.

It is exposed alongside the compile-and-run harness, returning the **same** `ParseOutcome` (so the two
are directly diffable — that diff is the equivalence gate):

```rust
use pgen::parse_harness_interpreter::{interpret_parse, InterpretOptions};
use std::path::Path;

let outcome = interpret_parse(
    Path::new("grammars/json.ebnf"),
    r#"{"a": [1, true, null]}"#,
    &InterpretOptions::default(),   // .entry_rule = Some("rule") to start from an alternate symbol
)?;
assert!(outcome.accepted);
// outcome.furthest_position — the deepest byte reached (the reject locus on !accepted)
// outcome.ast_json          — the typed AST on accept, byte-identical to the generated parser's
```

There is also a feature-independent core, `interpret_parse_gen_ast`, that takes an already-normalized
gen-AST (the triple `grammar_tree` / `rule_order` / `annotations` that codegen consumes) — the entry
point the `PARSE-HARNESS.5` differential-equivalence gate drives.

### Why it is trustworthy — "by verification" (the hard case)

Because the interpreter is a second implementation, its trust must be *earned*, not assumed. Two design
legs make it so:

1. **A minimized trusted surface (the shared core).** A generated parser is ~9 000 lines per grammar,
   but its *combinator control-flow is fully inlined per-rule as codegen templates* — there is no shared
   runtime function for it — while the *semantic + type layer* survives as callable runtime. So the
   interpreter **reuses verbatim** the shipped `ParseNode` / `ParseContent` types (so the AST it
   serializes is byte-identical), the quantifier-bounds decoder, the semantic-runtime checkpoint /
   rollback that speculation needs, and the gen-AST IR + loader; and it **re-expresses** only the small,
   parser-agnostic lexical/speculation primitives (`match_string`, `match_regex`, the layout consumers,
   `try_parse` — mirrored byte-for-byte from a generated parser's emitted code), plus the combinator
   dispatch (the ordered-choice tournament + `branch_policy`, sequence, the quantifier loop, lookahead)
   and the return-annotation fold. The residual divergence surface is therefore just that thin dispatch
   layer.
2. **Differential equivalence (the certifying oracle).** The residual is checked against the
   authoritative generated parser, byte-for-byte. The interpreter's tests assert it is **byte-identical
   to the registered `json` parser** (verdict + typed AST, across accept and reject inputs — exercising
   multi-branch ordered choice, sequence, regex tokens, rule-reference recursion, the object/array/spread
   return-annotation fold, and layout) **and to the compile-and-run harness on synthetic per-combinator
   grammars** (the A2.3 fixed-terminal-prefix shape, `*` / `+` repetition, optional `?`, and `&` / `!`
   lookahead) — verdict, `furthest_position`, and typed AST all identical. That synthetic-grammar diff is
   a *mini* version of the `PARSE-HARNESS.5` gate, whose oracle is the compile-and-run harness above.

### One subtlety worth calling out: interning to `'static`

The shipped `ParseNode.rule_name` (and a quantifier node's label) is a `&'static str` — a compile-time
literal in a generated parser. An interpreter over an *arbitrary* grammar has runtime `String`
rule-names, so to reuse the exact `ParseNode` type (and thus produce a byte-identical serialized AST) it
**interns** the finite set of grammar-derived strings — rule names, the handful of quantifier labels,
the annotation literals — leaking each distinct string to `&'static str` at most once, ever. For a
probe/gate tool that bounded, one-time-per-string leak is deliberate and the price of type-level
byte-identity.

### Honest bounds (the interpreter)

This is the interpreter **core**. It is byte-identical to the generated parser on the **structural +
return-annotation** surface — ordered choice under the default `longest_match` policy, sequence, the
quantifier forms, lookahead, terminals / regex-tokens / layout, rule references, and the full
return-annotation fold — plus the accept/reject **verdict** and `furthest_position`, proven on the smoke
set. Deferred to `PARSE-HARNESS.5`/`.6` (threaded here so they extend without restructuring, but not in
the `.4` smoke set): the **semantic-directive orchestration** that *gates parse outcomes* on the store
(`@predicate` gates, `@emit_fact`/scope effects), the non-default `branch_policy` / per-branch
`@priority` paths, packrat **memoization** (a transparent cache — AST-invariant — added where the full
corpus needs it), and the full-corpus, all-registered-grammars byte-identity. As with every claim on
this platform, the honest statement is *divergence-free over the tested corpus with a shared core* — not
a formal all-inputs proof.

## The differential-equivalence gate

The interpreter is a *second* parsing implementation, so its trust is **earned, not assumed**. The
mechanism that earns it is the **differential-equivalence gate** (`PARSE-HARNESS.5`, module
`rust/src/parse_harness_equivalence.rs`, run via `make -C rust parse_harness_equivalence_gate`): for each
registered grammar it runs **both** the interpreter (approach 1) and the shipped **generated parser** over
one deterministic corpus and asserts they agree **byte for byte** — same accept/reject verdict and, on
accept, the **byte-identical typed AST**. Any divergence names the grammar, the exact input, and where the
two serialized ASTs first differ.

**The corpus is the grammar's own strongest in-tree oracle of valid inputs:** its **stimuli generator**,
seeded at the standard `0` / `7` / `42`, over a bounded depth ladder, plus first-half **truncation probes**
for reject-path parity. Crucially, the interpreter and the stimuli generator consume the *identical*
normalized gen-AST that codegen consumed to build the generated parser — so all three implementations are
driven from one source of truth. Two robustness details make the gate trustworthy and non-flaky:

- **Bounded generation.** Corpus generation uses the same deterministic step budget the certificate-coverage
  pass uses (`generate_many_bounded`), so a super-linear grammar cannot hang the gate — the cutoff is
  machine-independent, so a seeded run yields the same corpus every time.
- **Large-stack workers.** The differential runs on a 512 MiB-stack worker thread, because recursive-descent
  parsing (both implementations) can nest deeply on a pathologically-nested sample and would otherwise
  overflow the small default stack and *abort the process* before the interpreter's logical depth guard
  fires (the "bound the *real* stack, not just the logical depth" discipline).

### The honest three-way classification (no silent caps)

The interpreter is not yet byte-identical on *every* registered grammar, and the gate says so honestly
rather than quietly testing only the easy ones. Every registered grammar is classified **exactly once**
(a completeness test enforces this, so a newly-added grammar cannot be silently unmeasured):

| Class | Grammars | Gate behaviour |
|---|---|---|
| **CERTIFIED** | `json`, `semantic_annotation`, `rtl_frontend`, `vhdl`, `systemverilog` (sv_2017), `scratch`, `regex`, `systemverilog_preprocessor`, `ebnf`, `return_annotation`, `rtl_const_expr` | must stay byte-identical — a regression **fails** the gate |
| **DEFERRED** | *(empty)* | the ratchet stays wired for a future newly-registered divergent grammar; every grammar deferred so far has been **promoted** (`.5.1`–`.5.5`) |
| **EXCLUDED** | `builtin_return_annotation`, `builtin_semantic_annotation` | out of scope *by construction* — their registry oracle is not a codegen parser of their own grammar (one aliases the `return_annotation` parser; the other uses a hand-rolled bootstrap parser), so the differential's premise does not hold |

`regex` and `systemverilog_preprocessor` were **promoted** from DEFERRED to CERTIFIED by `PARSE-HARNESS.5.1`
(session #42) — see *Four fidelity dimensions the interpreter mirrors* below for the four interpreter gaps
that were root-caused and closed. The promotion ratchet did its job: fixing `regex` also made
`systemverilog_preprocessor` byte-identical, the gate **failed** demanding the promotion, and it was
promoted the same commit (progress is never lost silently). `ebnf` was promoted by `PARSE-HARNESS.5.2`
(session #43) — see *Comment layout is grammar-specific* below. `return_annotation` was promoted by
`PARSE-HARNESS.5.3` (session #44) — see *Canonical serialization of the LR-chain blob* below. `rtl_const_expr`
was promoted by `PARSE-HARNESS.5.5` (session #45) — see *A curated corpus for un-generatable grammars* below —
which emptied the DEFERRED list: all 11 differentiable registered grammars are now CERTIFIED byte-identical.

### A curated corpus for un-generatable grammars (`PARSE-HARNESS.5.5`)

The gate's corpus is normally the grammar's own **stimuli generator** at fixed seeds over a bounded depth
ladder (`[6, 12, 18]`). That works for almost every grammar — but `rtl_const_expr` is a pathological case
for *unbiased* generation. Its expression core is a **~16-level precedence cascade**
(`rtl_const_expr → conditional_expr → logical_or_expr → … → multiplicative_expr → unary_expr → primary_expr
→ literal → decimal_integer`), where each level has the shape `X := Y (op Y)*`. Two things follow, both
tool-established via the CLI generation sweep (`ast_pipeline grammars/rtl_const_expr.ebnf --generate-stimuli
--max-depth D`):

- the generator needs `max_depth ≳ 30` **just to reach a leaf** — at depths 6–28 it fails outright
  (`Stimuli generation depth exceeded max_depth=N while expanding 'multiplicative_expr'`), so the gate's
  shallow ladder yields **zero** samples; and
- once it *does* have that depth, the ten `(op Y)*` quantifier levels each re-descend the whole chain, so
  generation is **super-linear**: the only working window is a razor-thin band around depth 32 (which emits
  giant, thousands-of-character expressions), and depths ≥ 40 **hang**.

So the stimuli generator cannot supply this grammar a usable differential corpus. The fix is a general,
parser-agnostic **curated-input corpus** (`CURATED_CORPUS`, keyed by grammar name alongside the
CERTIFIED/DEFERRED/EXCLUDED lists): a small, construct-complete, hand-authored set of *inputs* covering both
literal kinds, plain/dotted/package-qualified identifiers, all four unary ops, every binary op at each of the
ten precedence levels, multi-term chains, mixed precedence, ternaries (flat and nested), parentheses,
whitespace variety, and near-miss rejects. Crucially these are **inputs only** — the differential still
compares the interpreter against the *authoritative generated parser*, which supplies the verdict and typed
AST, so a curated input carries **no expected-output "mirror" risk**; it merely gives the differential
something to compare. Over that corpus the interpreter is byte-identical to the generated parser (151 samples,
zero divergence), confirming the leaf's thesis: `rtl_const_expr` was a **corpus** gap, not an interpreter
divergence. (Note: the same narrow depth-32 window is why `rtl_const_expr`'s certificate-coverage gate is
tuned to `--max-depth 32` with a step budget — where it fully certifies 48/48 rules, `UNKNOWN=0`.)

### Canonical serialization of the LR-chain blob (`PARSE-HARNESS.5.3`)

When PGEN eliminates left-recursion from a rule (for example `return_annotation`'s
`property_access_expression := accessor_base '.' identifier` and its array-access sibling), it rewrites the
rule into a base + suffix pair and attaches a synthetic `_pgen_lr_chain` value that carries a **`wrapper_specs`**
field — a JSON string holding, per alternative, the original rule's return-annotation *template*. That blob is
purely internal plumbing: it is only ever deserialized back to replay the rule's annotation over the folded
chain. Its **key order is semantically irrelevant**.

The template is a `UnifiedReturnAST::Object` whose properties were stored in a standard hash map, and the blob
is produced by serializing that map. A hash map iterates in a per-instance, non-deterministic order, which
caused two independent serializations to disagree byte-for-byte: the code generator **froze one arbitrary
order** into the generated parser as a string literal, while the interpreter **re-serialized a fresh order**
(itself varying run-to-run, and even sample-to-sample within one run) each time it loaded the grammar. Both
were "correct" — same content, different key order — but the differential-equivalence gate compares the typed
AST *byte-for-byte*, so the interpreter diverged from the generated parser on exactly the dotted/array
LR-eliminated shapes (`$1.S15`, `$1[$1*]`).

The fix canonicalizes the serialization at its single source of truth: the object's properties are now emitted
with **sorted keys** everywhere the type is serialized — the gen-AST blob, the generator's frozen literal, and
the interpreter's in-process re-serialization all agree. This is a shared, parser-agnostic primitive (it lives
on the `UnifiedReturnAST` type, used by every grammar), and it additionally removes a latent
non-determinism in the code generator itself: two `--generate-parser` runs now produce a **byte-identical**
`wrapper_specs` blob, where before they could freeze different (behaviorally-equivalent) orders. The only two
grammars whose LR-eliminated rules carry object-shaped templates — `return_annotation` and `semantic_annotation`
— were regenerated; both are byte-identical in the gate.

### Comment layout is grammar-specific (`PARSE-HARNESS.5.2`)

PGEN's EBNF meta-grammar offers three comment conventions — `#`-to-end-of-line, `//`-to-end-of-line, and
`/* … */` block comments — which the generated parser normally skips as **layout** (trivia) between tokens.
But a grammar can also define a *real token* that begins with one of those introducers: SystemVerilog's `#`
(delays / parameter lists), VHDL's `#` (based-literal delimiters), or — the case that motivated this leaf —
the EBNF grammar's own `block_comment := "/*" block_comment_content "*/"`, which makes `"/*"` a genuine
grammar token, not trivia.

When that happens, skipping the introducer as layout would **steal** the token, so codegen **suppresses that
comment arm** for that grammar (a per-introducer static decision, GRAMMAR-WELLFORMED.H.11.5). The result is
grammar-specific: `json` skips all three; `regex`/`vhdl`/`systemverilog`/`rtl_frontend` do **not** skip `#`;
`semantic_annotation` skips only `#`; and **`ebnf` does not skip `/* … */`** — so the generated ebnf parser
*rejects* a comment-only input like `/**/` (it must be matched structurally, and the top-level rule requires
real grammar content).

The interpreter had been skipping **all three** introducers unconditionally, so it *accepted* `/**/` where
the generated ebnf parser rejected it. The fix makes the interpreter consult **codegen's own suppression
predicate** — the exact function the code generator uses to decide which arms to emit — so the interpreter's
two layout skippers gate each comment arm identically to the generated parser, for **every** grammar. Because
the interpreter now computes the same decision from the same source of truth, it can never drift: a change to
codegen's rule is automatically reflected, and a dedicated gate test pins the per-grammar `(#, //, /*)` matrix
against the shipped parsers. This is the same discipline as the `.5.1` fidelity work — mirror the generator
expression-for-expression, keyed on a grammar *property*, never on a grammar name in the interpreter's logic.

Notably, the CERTIFIED set includes the **store-using** SystemVerilog / VHDL / rtl_frontend — the
interpreter is byte-identical to their generated parsers over this corpus even though it does not yet fully
orchestrate store-gated *parse outcomes*. That is an honest **corpus-scoped** certification (byte-identical
over this deterministic corpus), not an all-inputs proof; the deeper store-gated-outcome constructs and a
combinator-complete corpus are the job of the `.6` combinator+semantic suite and the `.7` fuzz lane.

### Four fidelity dimensions the interpreter mirrors (`PARSE-HARNESS.5.1`)

Certifying `regex` — a whitespace-sensitive, built-in-using, transform-carrying, profile-gated grammar —
forced the interpreter to reproduce four codegen decisions it had previously glossed over. Each is a
**general** primitive (keyed on a grammar capability, never a grammar name in the interpreter's own logic)
that mirrors the shipped code-generator expression-for-expression, so it improves fidelity for *every*
grammar with that shape — not just regex:

- **Whitespace-sensitivity (a per-grammar layout policy).** Most grammars skip inter-token layout; a few
  (`regex`, `systemverilog_preprocessor`) are whitespace-*sensitive* and must not. Codegen encodes this as
  three boolean layout flags; the interpreter now derives the identical `LayoutPolicy` and gates its layout
  consumers on it. Without this, the interpreter skipped a literal space inside `\Q]\E* ?` and bound the
  `?` as a lazy quantifier suffix where the generated parser leaves it empty.
- **Unresolved-reference built-ins.** A grammar may reference a rule that has no definition because it is a
  codegen-native matcher (`builtin_any_char`, `builtin_ascii_char`, `true`/`false`, a semantic-annotation
  hook). The interpreter now resolves those references to the same native matchers instead of erroring —
  which is what lets `regex`'s `unicode_char = !builtin_ascii_char builtin_any_char` match non-ASCII input.
- **`@transform` span coercion + a post-parse contract.** A `@transform` annotation coerces a matched span
  to a typed value (e.g. `digits` → the integer `12`, rendered as a JSON number, not a digit array). And
  some registered parsers layer a **post-parse semantic contract** on top of the grammar parse — `regex`
  applies PCRE2-fidelity validation (rejecting e.g. `$+`, a quantifier on an anchor, which the grammar
  accepts but PCRE2 does not). The gate applies that same registry contract to the interpreter's verdict, so
  both sides are compared at the identical "grammar-parse + registry contract" layer; the interpreter core
  stays a pure grammar-parse reproduction (parser-agnostic).
- **`@profiles` dialect gating.** A rule tagged `@profiles: [...]` is active only under a matching dialect
  profile (regex's default is the strict `pcre2` profile; SystemVerilog distinguishes `sv_2017` / `sv_2023`).
  The interpreter now threads the active profile and Backtracks a profile-gated rule at entry when the active
  profile is not allowed — mirroring the generated parser's rule-entry profile guard.

## The structural combinator suite (`PARSE-HARNESS.6.1`)

The differential-equivalence gate above proves the interpreter byte-identical to the generated parser —
but only over the constructs the *shipped* grammars happen to use. To trust the interpreter on an
**arbitrary / synthetic** grammar (the whole point of the harness — e.g. the `a | ab` linter-soundness
probe), we need equivalence **per combinator**, in isolation. That is the **structural combinator suite**
(`PARSE-HARNESS.6.1`, module `rust/src/parse_harness_combinator_suite.rs`, run via
`make -C rust parse_harness_combinator_gate`).

It is a table of **small isolating grammars**, one per structural combinator the interpreter dispatches.
For every `(grammar, input)` pair it runs **both** the interpreter and the **compile-and-run oracle**
(approach 2 — the real codegen + runtime, authoritative *by construction*; the registry oracle the `.5`
gate uses does not apply because these grammars are synthetic and never registered) and asserts they agree
**byte for byte**: the accept/reject verdict, `furthest_position` on reject, and the typed AST on accept.
Because the corpus is a *fixed curated input set* (not the seeded stimuli generator), the differential is
`synthetic grammar × curated input` with no randomness — deterministic by construction.

The 20 isolating cases cover the whole structural surface:

| Combinator | Isolating grammar (essence) | What it proves |
|---|---|---|
| ordered choice — **default `longest_match`** | `start := "a" \| "a" "b"` | picks the longer alt on `"ab"` → **accept** |
| ordered choice — **explicit `longest_match`** | `@branch_policy: longest_match` + same | the explicit path matches the default |
| ordered choice — **`ordered`** | `@branch_policy: ordered` + same | picks the *first* alt on `"ab"` → leaves `"b"` → **reject** |
| ordered choice — **`priority_first`** | `@branch_policy: priority_first` + `@priority: [1,2]` | picks the higher-`@priority` alt (reorders vs source) |
| **always-succeeds** (`e? \| keyword`) | `start := opt \| kw`, `opt := "x"?` | the always-matching `opt` does *not* shadow `kw` under `longest_match` |
| **sequence + backtrack** | `start := "a" "b" \| "a" "c"` | backtrack after the shared `"a"` prefix |
| quantifier **`?`** / **`*`** / **`+`** | `item?` / `item*` / `item+` | optional / zero-or-more / one-or-more |
| quantifier **`{n}`** / **`{n,m}`** / **`{n,}`** / **`{,m}`** | `item{2}` / `item{2,3}` / `item{2,}` / `item{,2}` | the four bounded windows enforce exactly their `(min, max)` — first-class since `BOUNDED-QUANT.1` |
| quantifier **zero-length guard** | `start := item*`, `item := "x"?` | a `*` over a nullable element must stop at the first empty iteration |
| lookahead **`!`** / **`&`** | `!"x" any` / `&digit rest` | zero-width negative / positive lookahead |
| **atom** — terminal / regex-token | `"hello"` / `/[0-9]+/` | exact literal / anchored pattern match |
| **rule reference** | `start := a b` | dispatch to referenced rules |
| **left recursion** (LR-eliminated) | `expr := wrapper \| term`, `wrapper := expr "+" term` | the wrapper form is rewritten to `base (suffix)*` |

The gate also asserts a **completeness** invariant (every combinator in the enumerated universe has ≥1
case, and every case name is unique — no silent gap), and folds in the load-bearing **A2.2/A2.3
discrimination proof**: on the *same* `a | ab` grammar, `longest_match` **accepts** `"ab"` while `ordered`
**rejects** it — proven on **both** the interpreter and the real generated parser, in agreement. That is
precisely the fact that makes an *unconditional* `FixedTerminalPrefix` shadowing verdict *unsound* under
PGEN's backtracking engine, and it is why the harness was built. (This evidence was consumed by
`GRAMMAR-WELLFORMED.A2.3`, 2026-07-07 — the harness's first real use: the linter's fixed-prefix verdict is
now **policy-conditional**, firing only under `@branch_policy: ordered`; see the *Grammar
well-formedness* chapter.)

### Two tool-established subtleties this suite surfaced

Building the suite exposed two behaviours worth knowing when authoring grammars — both found by the oracle,
not by guesswork:

- **Bounded quantifiers `{N}` / `{N,M}` / `{N,}` / `{,M}` were half-wired — CLOSED by `BOUNDED-QUANT.1`
  (2026-07-07).** As landed, the suite established that the EBNF *frontend* parses `item{2}` into a
  quantifier node and the shared runtime honours the bounds, but **codegen aborted** with
  `Unknown quantifier: 2` — while the *stimuli generator* happily generated strings for the same grammar
  (a generator⟷parser duality break). The root cause was a dialect split on the quantifier surface
  string: the frontend emits the brace-**stripped** raw-AST token (`["quantifier","2"]`), the canonical
  `parse_quantifier_bounds` decoder spoke only the braced spelling, and the stimuli generator bypassed it
  through a private duplicate parser that spoke brace-less. `BOUNDED-QUANT.1` taught the ONE canonical
  decoder both spellings and delegated the stimuli duplicate to it; the four `quant_bounded_*` cases in
  the table above are the per-combinator differential proof that all four bounded windows now compile,
  parse, and interpret byte-identically.
- **LR-elimination shifts the canonical entry.** When PGEN eliminates the wrapper/indirect left-recursion
  form, it **prepends** the synthetic `_lr_base` / `_lr_suffix` helper rules to the rule order — so
  `rule_order[0]` is no longer the semantic entry (`expr`); it becomes the base rule. Driving an
  LR-eliminated grammar therefore requires naming the real entry explicitly (the suite does, applying the
  same entry to both the interpreter and the oracle so the differential stays valid). This only affects
  synthetic left-recursive grammars — the shipped grammars express operator chains iteratively
  (`X := Y (op Y)*`), which needs no elimination.

Bare *direct* left recursion (`A := A x | y`) is a deliberately **out-of-scope** case: PGEN's structural
elimination only matches the wrapper form, so direct recursion is left to *runtime cycle-breaking*
(`RecursionGuard`). On that path the interpreter and the generated parser agree on the verdict but diverge
on `furthest_position` (measured: interpreter reaches `2`/`4`, the generated parser stays `0`) — a genuine
interpreter-fidelity gap kept as a durable, re-runnable probe and surfaced for a follow-up, distinct from
the LR-*eliminated* combinator the suite certifies.

## The semantic-directive orchestration suite (`PARSE-HARNESS.6.2`)

The structural suite above proves the interpreter per-combinator — but PGEN parsing is not purely
structural: a grammar can **gate parse outcomes on the semantic store** (`@predicate` in the `pre` /
`branch` / `post` phases), **populate** that store (`@emit_fact`, `@open_scope` / `@close_scope`), resolve
`$references` against the parsed content, and interact with memoization and speculative rollback. This was
the interpreter's explicitly-deferred, least-proven surface (its honest bound since `.4`). The
**semantic-directive orchestration suite** (`PARSE-HARNESS.6.2`, module
`rust/src/parse_harness_semantic_suite.rs`, run via `make -C rust parse_harness_semantic_gate`) closes it
the same way `.6.1` closed the structural half: small isolating grammars, one per orchestration construct,
each differentially verified **byte-identical** (verdict + `furthest_position` + typed AST) against the
compile-and-run oracle over fixed curated inputs — deterministic by construction, report-first, with a
completeness invariant so no construct can be silently unmeasured.

Landing the suite also landed the capability it certifies: the interpreter now runs the **full semantic
orchestration mirror**. The queries and the store itself were always the *shared runtime* (reused
verbatim: predicate evaluation, transactions, checkpoints, deltas, the scope tree); what the interpreter
mirrors from the emitted codegen templates is the orchestration *glue* — the rule-level transaction
skeleton (PRE gates → body → effects → library imports → POST gates → library exports → commit, with a
named rollback on failure), the tournament's per-branch C3-B semantic-delta capture/rollback with a
winner-only replay, branch-phase predicates, winning-branch branch-start inline actions, the `$reference`
resolver family (named / dotted / `view: raw` vs `view: shaped` / `.len`), and the **split packrat memo**
with semantic-delta replay (the transaction *wraps* the memo, so a rule's own gates and effects are
re-evaluated fresh on every memo hit; only the body is cached — and since MEMO-STORE-SOUNDNESS.2 the
memo is **taint-gated with write-epoch validation**: a body that transitively consulted the store is
cached *epoch-stamped* and replayable only while the store is unchanged, so a same-position retry
after a store change evicts the stale entry and honestly re-parses).

The 24 isolating cases cover the orchestration surface:

| Construct | Isolating grammar (essence) | What it proves |
|---|---|---|
| **post gate** (hit + miss) | `@emit_fact` at `decl`, `has_fact` post gate at `use` | declare-before-use accepts; use-of-undeclared rejects |
| **pre gate** | `has_fact` with `phase: pre` | blocks rule *entry* before the body runs |
| **inline branch gate** | `pick := @predicate{…, phase: branch} alt1 \| alt2` | an inline branch predicate is **branch-local** (gates only its own alternative) — the store flips *which* branch wins |
| **branch-local selection** | helper rules with `phase: post` gates per alternative | the store flips *which* branch wins — same verdict, different AST (the SV idiom) |
| **attribute gate** | `fact_attribute_equals` over two emit families | same-name fact with the wrong attribute still rejects |
| **negative gate** | `lacks_fact` | accept-when-absent, reject-when-present |
| **counting gate** | `fact_count_at_least(item, 2)` | the running per-kind count gates the parse |
| **scope visibility** | `@open_scope`/`@close_scope` + `has_fact_in_current_scope` | a fact emitted inside a closed scope is invisible at the outer depth |
| **scope identity** | `current_scope_is(class, "c")` | the open scope's kind/name is verdict-observable |
| **C3-B rollback** | a *successful but losing* branch emits | only the WINNING branch's emissions survive the tournament |
| **zero-length emit** | `maybe := "x"?` with `@emit_fact`, under `*` | a zero-length success fires its effects even though the iteration is structurally discarded |
| **raw named walk** | `$word` on rules without `->` | the recursive named-descendant walk over the raw tree |
| **positional `$N` resolution** | `name: $2`, `$2.word`, `$2[0][2]` in directive payloads | positional refs **resolve** since POSITIONAL-PAYLOAD-REFS.2 — plain, dotted (self-match on the wrapped rule), and chained-indexed (`[0]` unwrap + 0-based child) all live-covered |
| **positional hard-error parity** | `$3.word` where position 3 is the literal `"]"` | a dotted segment walking into terminal content stays unresolvable → the directive hard-errors and fails the rule, identically on both sides |
| **shaped view** | `$w.body` with `view: shaped` | dotted resolution against the `->`-shaped JSON |
| **`.len`** | `name: $body.len` | the resolved text's character count |
| **branch-start actions** | inline `@emit_fact` at a branch start | fires for the WINNING branch only (INLINE-ACTIONS.2) |
| **emit attributes from refs** | `kindattr: $kind.body` | attributes resolved from the parse + `fact_attribute_equals` over them |
| **library no-op parity** | `@export_to_library`/`@import_from_library`, no dirs configured | both sides skip identically (real I/O is registry-owned) |
| **memo × store (gate retry)** | a gated rule re-tried at the same position after a store change | the memo caches the BODY only; the rule's own gates re-evaluate fresh |
| **memo × store (tainted failure)** | an *unannotated wrapper* over a gated rule | a store-tainted body failure is cached *epoch-stamped* and **evicted** when the zero-width store change bumps the epoch — the retry re-parses fresh and accepts |
| **memo × store (tainted success, verdict)** | an unannotated `pick` over a gated *longer* branch vs a plain shorter one | a store-tainted tournament win is **evicted** once the store moves — the retry re-runs the tournament and the longer gated branch wins (the pre-fix stale replay rejected this input) |
| **memo × store (tainted success, tree)** | the equal-length twin with shaped `kind` markers | the retry re-runs the tie under the NEW store — the byte-identical AST comparison pins the winner on both implementations |
| **quoted name args** | `args: [mode, "special"]` vs a `$2`-emitted fact; `current_scope_is(block, "sc")` vs a `$2`-named scope | quoted String args match Identifier-coerced `$ref` names **textually** since FACT-NAME-MATCHING.2 — the F2 dead-gate class closed on both unified sites |

### Grammar-author facts this suite established (tools-first)

Each of these was found by the oracle or a scratch-slot trace while building the suite — they are
behaviors of the *shipped engine*, now pinned differentially and worth knowing when writing grammars:

- **Quoted and unquoted name args are equivalent** (since `FACT-NAME-MATCHING.2`, 2026-07-06): fact
  and scope NAME matching is **textual** — `String("x")`, `Identifier("x")`, and a `$ref` resolving
  to `x` all match each other (case-sensitively), the same scalar-friendly semantics attribute
  values always had. *History:* name matching used to be variant-strict (the fact index keyed on
  the raw value enum — an accident of the performance-index commit), so a *quoted* `"x"` in `args:`
  could never match a `$ref`-emitted `Identifier("x")` (`has_fact(kind, name=String(...)) → false`
  with the fact present — a silently dead gate this suite found); SV's grammars use unquoted
  identifiers throughout because of that era. The unification covered both strict sites (the fact
  index and `current_scope_is`) in the shared runtime, so the interpreter inherited it automatically.
- **An inline `phase: branch` predicate is branch-LOCAL** (gates only the alternative it is attached
  to) — matching the semantic-annotation book's published contract. *History:* until
  `BRANCH-PREDICATE-LOCALITY.2` (2026-07-06) the registry accessor flat-mapped every branch bucket, so
  an inline branch predicate was broadcast rule-wide (and double-evaluated on its own branch); the
  suite found the discrepancy, the fix restored the introducing commit's own "candidate branch only"
  semantics, and `sem_branch_gate` was deliberately re-anchored to pin branch-locality. Helper rules
  carrying `phase: post` gates (the SV `net_declaration` pattern) remain an equivalent idiom.
- **Positional `$N` references work in directive payloads** (since `POSITIONAL-PAYLOAD-REFS.2`,
  2026-07-06): `name: $2` resolves position 2's text, `$2.word` walks a dotted segment from a
  position, and `$2[0][2]` chains indexed access. The raw-tree walk facts an author needs: positions
  are 1-indexed and **literals occupy positions**; a position that binds a rule **wraps the rule
  node**, so a dotted segment may self-match its name, and indexing into the rule's children needs
  the `[0]` unwrap first (`$2[0][2]`, not `$2[2]`); an unresolvable segment (e.g. one walking into a
  literal element) is a **hard error** that fails the rule. *History:* the annotation compiler used
  to strip the `$` sigil (`$2` froze as `RuleReference("2")`, which the *named* lexer rejects on the
  digit head) — the deliberately-built positional resolver was unreachable from compiled directives,
  a half-wired surface found by this suite; the fix preserves the sigil for digit-headed refs at the
  single shared payload-capture point, so the interpreter inherits it automatically.
- **The memo is taint-gated with write-epoch validation (stale store-dependent entries can never
  replay).** The memo key is `(rule, position)` — store-blind — so replaying a store-dependent outcome
  after the store changed would resurrect stale verdicts (and stale *trees*). Since
  MEMO-STORE-SOUNDNESS.2 every `memoized_call` snapshots the store's predicate-evaluation counter
  around the rule body: if the body (transitively — nested rules' gates, inline branch predicates,
  composed defs) evaluated ≥1 predicate, the outcome is cached **stamped with the store write epoch**
  (mutations only: emissions, imports, scope opens/closes, discarding rollbacks) and is replayable
  only while that epoch is unchanged — predicates are pure functions of position-determined args plus
  the store, so an unchanged epoch reproduces every verdict; a moved epoch evicts the entry and the
  retry re-parses fresh. Outright *exclusion* of tainted outcomes was measured at **117× slower** on
  SV (predicates fire on virtually every identifier path — exclusion guts packrat protection), which
  is why validation, not exclusion, is the shipped design. HISTORY: the pre-fix cache was store-blind
  on both sides — a stale failure replayed a REJECT of valid input (`sem_memo_wrapper`, session #47)
  and a stale success replayed old tournament content that could flip the verdict *and* the tree
  (`MEMO-STORE-SOUNDNESS.1`, session #49) — both now deliberately re-anchored to the sound behavior.
  A rule's *own* gates still never taint its own entry (they evaluate in the transaction, outside the
  memo, fresh on every hit); predicate evaluation is the store's ONLY read path into parsing, so the
  counter is a complete store-dependence signal.
- **A discarded zero-length iteration's effects persist.** A rule that succeeds matching zero bytes fires
  its `@emit_fact` (effects run on rule success), even when the quantifier's zero-length guard then
  discards the iteration structurally.
- **Unresolved-reference stubs have no rule preamble.** A referenced-but-undefined rule compiles to a bare
  `Err(Backtrack)` stub with no `furthest_position` bump, no memo, no rule context — the interpreter
  dispatches these before its own rule preamble to stay byte-identical.

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
- The interpreter (`PARSE-HARNESS.4`, core landed — see *The grammar-AST interpreter* above) is a
  genuine second implementation; its trust is *earned* by the differential-equivalence gate (`.5`, see
  *The differential-equivalence gate* above), and the honest claim is "divergence-free over the tested
  corpus with a shared core" — **not** a formal all-inputs proof. The gate now **certifies all 11
  differentiable grammars byte-identical** (including SystemVerilog / VHDL / rtl_frontend, plus `regex` and
  `systemverilog_preprocessor` since `.5.1`, `ebnf` since `.5.2`, `return_annotation` since `.5.3`, and
  `rtl_const_expr` since `.5.5` — via a curated corpus for that un-generatable grammar); the DEFERRED
  ratchet is now empty. The combinator-complete corpus has now also landed in full: the **structural**
  half (`.6.1`, *The structural combinator suite* above — 20 isolating grammars: 16 at landing, plus the
  four bounded-quantifier cases added when `BOUNDED-QUANT.1` closed that half-wire) and the
  **semantic-directive orchestration** half (`.6.2`, *The semantic-directive orchestration suite* above —
  24 isolating grammars covering the store-gated-outcome surface (20 at landing, since grown by the
  findings-driven re-anchors), which also landed the interpreter's semantic orchestration mirror +
  split memo). A fuzzing lane (`.7`) remains the optional push toward
  exhaustive. Out of harness scope on the semantic side: bootstrap facts (the cross-file `veer` surface)
  and real library I/O (both registry-owned), and coverage recording (a cert surface). See
  `docs/tasks/PARSE-HARNESS.md` §3.3 / §3.4 / §21.3.

## See also

- [Grammar Well-Formedness & Well-Definedness](grammar-wellformedness.md) — the linter-soundness
  questions the harness exists to answer empirically (e.g. ordered-choice shadowing under backtracking).
- [Diagnostic & Debug Toolbox](diagnosing-unknowns.md) and
  [Debugging With `parseability_probe`](parseability-probe-debug.md) — the toolbox the scratch slot
  makes available on synthetic grammars.
- `docs/tasks/PARSE-HARNESS.md` — the full design, trust architecture, and phasing for all three
  approaches.
