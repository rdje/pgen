# Developer Architecture

Once you move past user-facing commands, the next step is understanding how the Rust-first platform is organized.

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
