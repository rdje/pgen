# Developer Architecture

> **Part II · Inside PGEN.** From here on, the book is for contributors who modify
> PGEN's internals. It explains *how the engine works* in depth (in prose, not pasted
> Rust — the [Source Map](source-map.md) points to the code for the last 100%). If you
> only *use* PGEN, you do not need Part II — Part I has everything you need.

Once you move past user-facing commands, the next step is understanding how the Rust-first platform is organized.

## Parsing Model — where PGEN stands (PEG, Packrat, data-dependent)

PGEN is a **packrat-memoized, data-dependent (stateful) Parsing Expression Grammar (PEG)
parser-generator** — in the same family as SPEG / Nez / data-dependent grammars, with
automatic left-recursion elimination, synthesized-attribute return annotations, and a
semantic store. The three terms:

- **PEG** (Ford, POPL 2004): a *recognition-based* grammar. Alternatives use **ordered
  choice** — the first matching alternative commits, so a PEG is **never ambiguous** (unlike
  a CFG used by yacc/bison/ANTLR). It has greedy/possessive repetition and **syntactic
  predicates** `&e` / `!e` (unlimited lookahead that consumes nothing). PGEN takes the PEG
  *determinism* property — every choice has exactly one winner, so a grammar is never
  ambiguous — but **selects that winner by a branch tournament, not by first-match commit**.
  ⚠️ **Do not read `|` as "first alternative wins":** the default `@branch_policy` is
  `longest_match` (every alternative is tried and the one consuming the most input wins;
  ties go to the earlier alternative). Classical PEG first-success commit is available, but
  only when a rule asks for it with `@branch_policy: ordered`. Measured, on the discriminating
  input `start := "a" | "a" "b"` against `"ab"`: the default **accepts** (the longer
  alternative wins), `ordered` **rejects** (it commits to `"a"` and strands the `b`). The
  `&`/`!` predicates and the greedy/possessive repetition are faithful PEG.
  - This distinction is load-bearing, not pedantic: two `--lint-grammar` deadness verdicts
    were unsound until they were made policy-conditional, precisely because they reasoned
    "PEG commits to the first success". See *Grammar Well-Formedness* (the A2.2/A2.3/A2.4
    corrections), the per-policy row table in *The Parse Harness*, and the grammar-author's
    treatment in the `ebnf` parser book's *Rules and Expressions* chapter.
- **Packrat** (Ford, ICFP 2002): the **linear-time *implementation* of a PEG** via
  memoizing every `(rule, position)` result, which eliminates PEG's worst-case exponential
  backtracking. PGEN does this (its `MemoEntry` cache). Packrat assumes a **pure** parse
  function.
- **Data-dependent / stateful PEG** (SPEG, Nez, Yakker): a PEG extended with parse-time
  **state** — a symbol table the grammar consults to decide rules, enabling
  context-sensitivity (e.g. SystemVerilog *type-vs-expression* disambiguation). PGEN's
  **semantic store** (`@emit_fact` + `@predicate`-gated rules + scope tree) is exactly this.
  Because the store makes the parse function *impure*, PGEN attaches a **semantic *delta*** to
  each memo entry and replays it on cache hits (the published-correct fix, Laurent & Mens,
  SLE 2016).

**Left recursion is eliminated for you — direct, wrapper-chained and indirect alike.** The **AST
pipeline's `eliminate_left_recursive_patterns` pass** (on by default; toggle
`--eliminate-left-recursion`) runs three stages, each feeding the next:

1. **Direct inline recursion is normalized into the wrapper shape.** A flat
   `expr := expr "+" term | term` — the way every language standard's Annex A writes it — has its
   left-recursive alternatives hoisted into synthetic rules so the wrapper planner can see them
   (`normalize_direct_left_recursive_alternatives`, `GRAMMAR-WELLFORMED.A2.5`). Before that, such an
   alternative reached codegen intact and was *dead code* the runtime guard rejected.
   ⭐ Both this stage and stage 2 ask **one** question — *is this alternative left-recursive, inline
   or through a single wrapper hop?* (`alternative_is_left_recursive`) — and they used to answer it
   differently: stage 1 saw only the inline shape, stage 2 only the one-hop shape. A rule mixing the
   two, `expr := expr "+" term | mulwrap` with `mulwrap := expr "*" term`, therefore escaped **both**
   guards: it derives nothing, so it must be reported as non-terminating where the author wrote it,
   and instead the pipeline hoisted a synthetic rule and reported the error against *that* name
   (`ENGINE-UNIVERSAL-SERVICES.23`). Sharing the predicate is what makes the two stages agree by
   construction. The predicate is deliberately **one hop**, matching what stage 2 can actually
   eliminate — a longer cycle is stage 3's subject and `--lint-grammar`'s `left_recursion_unhandled`.
2. **Wrapper chains are eliminated** — `X := seed | W`, `W := X suffix` becomes
   `X := X_lr_base ( X_lr_suffix )*` (`detect_left_recursive_chain_plan`).
3. **Indirect routes are eliminated** — `X := … A …`, `A := … B …`, `B := … X …`, where the cycle
   spans many hops, becomes the same `X_lr_base ( X_lr_suffix )*` plus one sheared **clone** per
   intermediate on the route (`ast_pipeline::indirect_lr_elimination`,
   `ENGINE-UNIVERSAL-SERVICES.13`). The pass applies each plan to a copy and re-runs the
   left-recursion lint before committing, so a plan that would only half-shear a cycle is refused
   with the surviving path named rather than shipped.

Every stage's rewrite is **folded back through the author's return annotations** at AST
construction, so an eliminated rule returns the declared left-nested AST and is indistinguishable
from a hand-written one — one shared implementation (`ast_pipeline::lr_chain_fold`) called by both
codegen graphs and the interpreter (`ENGINE-UNIVERSAL-SERVICES.8`; see *The Annotation System*).
For an indirect route the hops' annotations are *composed* into a single per-iteration template, so
that one fold covers a whole route without changing.

⛔ **Three bounds worth knowing.** A hop with a residual and no return annotation is **refused** (its
undeclared value is the engine's default shaping of the whole alternative, which no template can
reproduce — annotate it). A cycle whose closing path re-enters a rule already on the route, or runs
through a nullable prefix, a quantifier or a group, is outside the route walk and is refused loudly.
And — the one that decides the most in practice — a rule can absorb a chain only if **nothing that
outlives the rewrite can be starved by its new greed**, transitively: an alternative that is a bare
reference forwards consumption as well as shape, so a holder that never names the base rule can
still lose. See [Grammar Wellformedness](grammar-wellformedness.md).

For operator grammars a **precedence cascade** — loosest→tightest `head tail tail*` levels with
named tail rules and `-> $1` passthroughs — remains the idiom that keeps the AST faithful to
operator precedence, and it is what PGEN's expression grammars use (e.g. `constant_expression`).
⚠️ It is no longer a *workaround* for a missing capability, and it is not free: a cascade puts a
residual after the rule it recurses on, which is exactly the shape that starves chain absorption.
That is why SystemVerilog's SVA property knot is still uneliminated
(`ENGINE-UNIVERSAL-SERVICES.15`) — though the same starvation also arises from the LRM's own
`casting_type := … | constant_primary` / `cast := casting_type ' ( expression )` factoring
(`.17`), so it is a property of the SHAPE, not of hand-writing. Write the cascade when you want the
precedence; write the flat rule when you want the engine to do the work.

Two consequences worth remembering:

- Because it's a **PEG**, *alternative order is meaningful* — an earlier alternative can
  shadow a later one (`a | ab` makes `ab` unreachable). PGEN ships a shadowing lint for this.
- Because the store makes packrat **stateful**, the textbook linear-time guarantee is *not*
  automatic — verifying and preserving near-linear time under state is an explicit,
  tracked correctness concern (see the parser termination work).

## Repository Layout

The canonical path inventory. This chapter is its **only** home: per the
[PGEN README Stability Policy](../../reference/PGEN_README_STABILITY_POLICY.md), the
README carries architecture *at a glance*, and the exhaustive inventory lives here.

All paths are **relative to the repository root** — the root may be moved or relocated
to another filesystem without affecting the project.

### Grammars and generated output

| Path | What it holds |
|---|---|
| `grammars/` | EBNF sources (`*.ebnf`) — the single source of truth for every deliverable parser |
| `grammars/builtin_return_annotation.ebnf`, `grammars/builtin_semantic_annotation.ebnf` | bootstrap-safe annotation grammar contracts that break the annotation-parser chicken-and-egg cycle |
| `grammars/scratch/scratch.ebnf` | the **parse-harness scratch slot** — a blessed throwaway probe grammar whose body you overwrite freely to run an arbitrary grammar's input through the whole `parseability_probe` toolbox. See [The Parse Harness](parse-harness.md) |
| `generated/` | pipeline-output artifacts (parser sources, AST JSON dumps, return-annotation inventories) consumed by compile-time includes. ⛔ **Not tracked in git** — regenerate with `make -C rust SHELL=/bin/bash regenerate_generated_parsers` (see [The Gate Flow](gate-flow.md)) |
| `rust/target/generated_logs/` | scratch generation/debug logs, kept out of `generated/` |

### Rust surfaces

| Path | What it holds |
|---|---|
| `rust/src/` | the Rust AST pipeline, generators, parser registry, embedding API |
| `rust/build.rs` | compile-time generated-parser include-path resolver; emits *relative* `include!(env!(...))` paths so clean checkouts and relocated worktrees never depend on absolute filesystem paths |
| `rust/scripts/` | executable quality gates and policy runners |
| `rust/config/branch_protection_policy.json` | the tracked minimum branch-protection required-check contract |
| `rust/test_data/grammar_quality/` | gate contracts, corpora, deterministic case manifests |
| `rust/docs/` | Rust-specific architecture/API/test docs |

### Phase S bootstrap crates

| Path | What it holds |
|---|---|
| `rtl_const_expr/` | standalone constant-expression parser/evaluator baseline crate, including dotted and package-qualified (`pkg::NAME`) identifier lookup. Paired with `grammars/rtl_const_expr.ebnf` |
| `rtl_frontend/` | synthesizable-RTL frontend baseline crate wired to `rtl_const_expr`. Paired with `grammars/rtl_frontend.ebnf` and a curated generated-contract gate |

⭐ The **capability detail** for these two crates — which SystemVerilog constructs are
covered, which contract version is released, what the curated manifest proves — is
deliberately not duplicated here. It moves with the code, so it lives with the code's
own contract: `docs/contracts/` and `docs/rtl_frontend_parser_book/`, with live status
in `docs/book/src/roadmap-and-live-status.md`.

### External corpora and stimulus sources

| Path | What it holds |
|---|---|
| `regex_corpus_bundle/` | PCRE2-first regex corpus acquisition/inventory; immutable upstream snapshots kept separate from normalized corpus/oracle outputs |
| `json_corpus_bundle/` | the external test-corpus surface for the `json` parser — a vendored JSONTestSuite snapshot plus a reproducible runner and a measured characterization |
| `stimuli/generators/anvil/` | **ANVIL** (submodule) — a random by-construction generator of synthesizable SystemVerilog, the independent stimulus source for the `rtl_frontend` / `rtl_const_expr` families. Its validity model is architecturally independent of any grammar, and it is byte-reproducible per `(seed, knobs)`. ⛔ Deliberately **not** under `stimuli/*/subs/`: everything there is a vendored *corpus*, and ANVIL is a *generator* |

### Language-reference workspaces

| Path | What it holds |
|---|---|
| `docs/systemverilog/2017`, `docs/systemverilog/2023` | SV LRM conversion workspaces |
| `docs/vhdl/2019` | VHDL LRM conversion workspace |
| `docs/verilog/2005` | Verilog LRM conversion workspace |
| `docs/tcl/md/tcl.md` | local Tcl syntax note for the pending SDC/Tcl-shaped PNR lane — reference input only, not a shipped grammar |

The extracted and profiled grammar snapshots those workspaces produce
(`grammars/systemverilog_2017_lrm_extracted.ebnf`,
`grammars/systemverilog_2023_lrm_extracted.ebnf`,
`grammars/verilog_2005_lrm_extracted.ebnf`, and the profiled synthesis artifacts) are
retained in `grammars/` for regeneration traceability.

### Documentation and tooling

| Path | What it holds |
|---|---|
| `docs/book/` | this book — the curated live mastery surface |
| `docs/contracts/` | downstream parser integration contracts, the issue-reporting protocol, the released-parser bug ledger |
| `docs/reference/` | normative specs, matrices, closure roadmaps, release and README policy |
| `docs/tasks/`, `docs/decisions/` | task trees (layer B) and durable decision records (layer C) |
| `tools/`, `perl/` | conversion/extraction workflows and the legacy EBNF-to-JSON path still used in the hybrid flow |
| `tests/` | test how-to and test guides |

## Core Areas

### Rust AST pipeline

This is where grammar AST transformation, parser generation, stimuli generation, and CLI flows come together.

### Generated artifact policy

⚠️ **Corrected 2026-07-30 (`README-POLICY.1`).** This section previously read
*"Generated artifacts are tracked on purpose"*. That is false and was measured so:
`git ls-files generated/` returns **0 files**. `generated/` is **untracked**, and a
clean checkout must run `make -C rust SHELL=/bin/bash regenerate_generated_parsers`
before the crate will build with `--features generated_parsers` — `rust/src/lib.rs:72,78`
include the two annotation parsers by literal path with no `has_generated_*` cfg, so
their absence is a hard rustc error rather than a disabled feature.

What *is* true, and what the stale sentence was reaching for, is the property below:
reproducibility does not come from tracking the artifacts, it comes from the generator
being deterministic.

For that reproducibility to hold, **code generation is deterministic**: regenerating a parser
from the same grammar source (to the same output path) produces byte-identical output, so the
"regenerate twice, compare SHAs" check is meaningful. This
requires the generator to emit every *derived* collection in a **canonical order** rather than in
`HashMap` iteration order, which is randomized per process. The per-rule semantic-directive
registries the generated parser builds (`directives_by_rule`, `branch_directives_by_rule`,
`fact_kinds`) are therefore emitted **key-sorted**. Without that, a grammar carrying many semantic
annotations — SystemVerilog (dozens of store-gated rules) or regex — would regenerate to different
bytes on every run *even though the parser's behaviour is identical*, defeating byte-identity as a
no-regression signal. The order is purely a source-reproducibility property: the runtime is
order-insensitive (the generated parser re-inserts these into its own map), so the canonical emission
changes nothing about how the parser parses.

### Bootstrap and architecture evolution

PGEN still carries history from earlier bootstrap phases, but the active direction is explicit: Rust-first, EBNF-backed, proof-first generation.

## The responsibility boundary and its guard

PGEN's engine serves every EBNF identically. The rule that keeps it so has a testable form
(`ENGINE-UNIVERSAL-SERVICES`, the frame): **a concern belongs to the engine iff it is objectively shared by all
EBNFs — a property of parsing, not of the language parsed; it belongs to the grammar iff it is a fact about one
language.** Both directions can be violated silently: a grammar can hand-compile around an engine gap (the
founding case — three SystemVerilog rules hand-flattened around left recursion while the linter reported the
gap as "handled by PGEN"), and the engine can absorb a language's words (a help string whose example is
`sv_2017`, an API that types a family's profiles by hand). Every gate stays green either way, which is why the
boundary is a doctrine and not a convention.

**`ENGINE-NEUTRALITY`** (`scripts/check_engine_neutrality.sh`, ~1 s on every commit) reads the engine for the
words of the languages it parses:

- **The forbidden vocabulary is derived, never typed.** On every run the grammars under `grammars/` are classed
  by `rust/test_data/grammar_quality/engine_neutrality_register_v0.json` as *language* (a parser built for users:
  SystemVerilog, its preprocessor, the LRM-extracted twins, VHDL, the RTL grammars) or *meta* (the engine's own
  languages: the EBNF meta-grammar, the semantic- and return-annotation languages, the example grammars). A
  language grammar's name, its `@profiles` values, its `kw_*` keyword tokens and its multi-word rule names are
  forbidden in the engine; a meta grammar's words are engine vocabulary by construction and are subtracted —
  `quoted_string` and `string_literal` are the EBNF frontend's own token types, not leaks.
- **The engine is every `rust/src` source the register does not class otherwise**, and it may carry none of the
  vocabulary in production code. Test modules and comments may name families freely: they are fixtures and
  prose, not behaviour. The allow-list is empty by design.
- **The non-engine classes are a closed population under a ratchet.** A *per-family* component may name its
  own family only — its name, its profiles, its rules — and it is classed by WHERE it lives:
  `rust/src/families/<name>.rs` or anything under `rust/src/families/<name>/` is that family's, with no
  register row to type (the SystemVerilog preprocessor and its IEEE 1800-2017 §22.14 keyword tables live at
  `rust/src/families/systemverilog/`, the regex nesting ceiling and PCRE2 contract at `families/regex.rs`; a
  family's own contract probe under `bin/` is the one component classed by an explicit row). The *registry*
  (the build-derived list of generated parsers), the *api* (the embedding surface), the *bin* (operator tools)
  and the *harness* (test suites) files each carry a ceiling of production-code tokens that only falls — a
  rise fails the commit, a fall is promoted deliberately, and every non-zero ceiling names the leaf that owes
  its zero. **Every ceiling is at zero since 2026-09-29**: `.112` derived the API's and the registry's
  enumeration from the artifacts; `.113` moved the preprocessor beside its family and derived the operator
  binaries' and the harness's tables.
- **What the engine owns of preprocessing is the STAGE, not the preprocessor.** `rust/src/preprocessing_stage.rs`
  is the one neutral interface: a family declares a `PreprocessingStage` on its hooks (its option keys, its
  deprecated CLI spellings, its `run`), and the engine hands it a request every front end can express — the
  ordered compilation unit, the include search directories, the predefined macros, the profile the expanded
  text will be parsed under, and the family's own options by key — and gets back expanded text with a source
  map, an event log and diagnostics in the engine's shapes. The CLIs reach a stage only by family name
  (`ast_pipeline INPUT --preprocess <family>`, `parseability_probe --parse <grammar> … --preprocess`), so a
  second language with a preprocessor adds a component directory and nothing to the engine.
- **Thirteen self-test arms run before the tree is read**, so a guard that has stopped refusing is caught by its
  own run.

How to read a breach: `ENGINE source <file>:<line> names the <kind> <token> of a language grammar in production
code` means the fact belongs in a grammar or in a per-family component, not in the engine; `<file> ROSE a → b`
means a classed file gained language knowledge it had been shedding. The full census, every hit by class,
position and kind, is `bash scripts/check_engine_neutrality.sh --census`.

**The ladder a new capability climbs** (director, 2026-09-29). When work on one grammar needs something the
EBNF could not express, it is tried in this order, and stepping down needs a measured reason naming the
second language that diverges: (1) a grammar directive the engine serves for every family (`@profiles`,
`@entry`, the annotations), (2) a neutral engine service — a property of parsing, not of the language
parsed, (3) a family component shaped to be lifted later. A "neutral" service witnessed by one language is
a family capability in the engine's clothes; the guard reads names, not concept flavour (`--define` passed
it while carrying one family's word, and became `--symbol`), so the ladder is a line in the task-acceptance
template rather than a gate.

The grammar-side direction — a grammar carrying what the engine should provide — is held by the earned
left-recursion verdict of `--lint-grammar` and `LR-ELIMINATION-RATCHET`, and owed a census of its own
(`ENGINE-UNIVERSAL-SERVICES.2`).

## Front-End Workbench Direction

One increasingly important architectural direction is that PGEN should become a front-end workbench, not only a parser emitter.

That means the architecture should increasingly support:

- shaped ASTs,
- optional lossless front-end fidelity where needed,
- semantic-bundle export,
- stable node ids,
- generated traversal helpers,
- and explicit handoff seams for downstream compiler, elaborator, and linter passes.

The detailed planning surface for that direction now lives in:

- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`

## Diagnostics: severity vs verbosity

PGEN keeps two orthogonal axes for messages, and they must never be confused:

- **Verbosity** (`TraceLevel`: `Low`/`Medium`/`High`/`Debug`) governs **informational**
  output only. It is gated by the active trace verbosity (`none` by default), via
  `pgen_trace!` / `pgen_trace_low!` / … and the `trace_log` sink. A breadcrumb suppressed
  at low verbosity is fine — it carries no severity.
- **Severity** (`Severity`: `Warning` < `Error` < `Fatal`) is **always emitted**,
  unconditionally, to stderr — via `pgen_warn!` / `pgen_error!` / `pgen_fatal!` and
  `emit_diagnostic`. **A warning/error/fatal is NEVER gated by a verbosity level**, because
  a severity message hidden behind verbosity is a silent failure.

This separation is a hard rule (a masked error once hid the dominant cause of the
SystemVerilog stimuli-coverage residual for an entire campaign). It is enforced by
`scripts/check_diagnostics_and_docpaths.sh` (run in the pre-commit hook and CI), which
fails if the always-on mechanism is removed or if an unambiguous severity is routed
through the verbosity-gated trace.

The same script also enforces a second, unrelated rule: **every repo-internal file path in
a live/maintained documentation surface must be repo-root-relative**, never a
checkout-specific absolute path that captures a local home directory (an absolute path
rooted at `<your-home>/.../pgen/grammars/foo.ebnf` is non-portable and breaks on any other
clone — write `grammars/foo.ebnf` instead). The
guarded surfaces are `docs/book/src`, `docs/contracts`, `PGEN_USER_GUIDE.md`, `README.md`,
`docs/tasks`, `docs/decisions`, `KNOWLEDGE_MAP.md`, `docs/knowledge`, and
the live book. Append-only history (`CHANGES.md`, `DEVELOPMENT_NOTES.md`) and
repo-external references (which point outside the repository and have no repo-relative
form) are deliberately out of scope.

### Error-by-reason classification

Generation failures are classified — never folded into an anonymous count — by the single
canonical `GenerationErrorReason` enum (`classify_generation_error`):

| Reason | Meaning | Class |
|---|---|---|
| `DepthExceeded` | hit `max_depth` (recursion-depth budget) | structural budget |
| `RuleVisitLimit` | hit `max_rule_visits` (per-rule visit budget) | structural budget |
| `TargetTimeout` | primary-entry generation timed out | time budget |
| `HelperTimeout` | helper-probe generation timed out | time budget |
| `Other` | any other failure, incl. expected PEG backtracking | residual |

The two **structural-budget** reasons are surfaced per-run in the target-drive summary and
in a once-per-run always-on warning, so coverage shortfalls caused by the generation
budgets are visible rather than masked. (Note: an expected PEG rule-attempt failure that
the generator handles by backtracking is *control flow*, not a severity error, and stays
informational.)

## Primary Source Docs

- `docs/reference/RUST_CODEBASE_ANALYSIS.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `docs/AST_GENERATOR_ARCHITECTURE.md`
- `docs/ast_transformation_pipeline.md`
- `docs/BOOTSTRAP_MODE_SPECIFICATION.md`
- `docs/EBNF_INCLUDE_SYSTEM.md`
- `docs/parser_architecture_evolution.md`
- `docs/TEST_INFRASTRUCTURE.md`

## Contributor Guidance

When changing implementation:

- keep generated and handwritten surfaces in sync,
- update user-facing docs when commands or contracts change,
- update continuity docs before commit,
- prefer executable proof over prose claims.
