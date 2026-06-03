# Developer Architecture

Once you move past user-facing commands, the next step is understanding how the Rust-first platform is organized.

## Parsing Model — where PGEN stands (PEG, Packrat, data-dependent)

PGEN is a **packrat-memoized, data-dependent (stateful) Parsing Expression Grammar (PEG)
parser-generator** — in the same family as SPEG / Nez / data-dependent grammars, with
automatic left-recursion elimination, synthesized-attribute return annotations, and a
semantic store. The three terms:

- **PEG** (Ford, POPL 2004): a *recognition-based* grammar. Alternatives use **ordered
  choice** — the first matching alternative commits, so a PEG is **never ambiguous** (unlike
  a CFG used by yacc/bison/ANTLR). It has greedy/possessive repetition and **syntactic
  predicates** `&e` / `!e` (unlimited lookahead that consumes nothing). PGEN is a faithful
  PEG: ordered choice, `&`/`!`, greedy quantifiers.
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

**Left recursion is handled for you — automatically.** You write the *natural*
left-recursive EBNF (e.g. `expr := expr "+" term | term`); the **AST pipeline eliminates it
at transform time** (the `eliminate_left_recursive_patterns` pass, on by default; toggle
`--eliminate-left-recursion`), handling direct and indirect/chained cases, plus a runtime
cycle-breaker. You do **not** rewrite grammars into tail-rule form by hand — that would be
impractical for real expression/operator grammars, and it would distort the AST and the
annotations.

Two consequences worth remembering:

- Because it's a **PEG**, *alternative order is meaningful* — an earlier alternative can
  shadow a later one (`a | ab` makes `ab` unreachable). PGEN ships a shadowing lint for this.
- Because the store makes packrat **stateful**, the textbook linear-time guarantee is *not*
  automatic — verifying and preserving near-linear time under state is an explicit,
  tracked correctness concern (see the parser termination work).

## Core Areas

### Rust AST pipeline

This is where grammar AST transformation, parser generation, stimuli generation, and CLI flows come together.

### Generated artifact policy

Generated artifacts are tracked on purpose. That makes clean-checkout validation and reproducible contract work possible.

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
