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

**Left recursion: indirect/chained recursion is eliminated for you; direct inline recursion
is expressed as a precedence cascade.** The **AST pipeline's `eliminate_left_recursive_patterns`
pass** (on by default; toggle `--eliminate-left-recursion`) rewrites *indirect / chained* left
recursion — where a rule reaches itself only through bare-reference wrapper rules — into a
non-recursive form, backed by a runtime cycle-breaker. **Direct inline left recursion
(`A := A op A | term`) is *not* auto-eliminated**: the detector
(`detect_left_recursive_chain_plan`) targets the indirect wrapper-chain shape, so a flat
inline rule like `expr := expr "+" term | term` yields zero transformations and its left branch
is blocked at runtime. For operator grammars you therefore write the natural **precedence
cascade** — loosest→tightest `head tail tail*` levels with named tail rules and `-> $1`
passthroughs — which is the proven idiom across PGEN's expression grammars (e.g.
`constant_expression`). The
cascade keeps the AST and annotations faithful to operator precedence; you do **not** hand-rewrite
a single flat rule into tail form (that would distort the AST and the annotations).

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
