# DIAG-SEVERITY — severity (warning/error/fatal) must never be gated by a trace/verbosity level

> Task tree. **Metadata** — Status: `active`; Created: 2026-06-02 (`PGEN-DIAG-SEVERITY-0001`);
> Roadmap lane: cross-cutting engine/diagnostics correctness.
>
> Director directive (2026-06-02, emphatic): *"warnings, errors and fatals shall never,
> ever, ever be masked by a trace level. Trace levels should solely apply to info
> messages, not to warnings, errors or fatals. Create a new task-tree to correct this
> in the entire codebase."* Principle recorded as
> [[feedback_severity_never_gated_by_verbosity]].
>
> Discipline: [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_ast_pipeline_parser_agnostic]], [[feedback_no_workarounds_fix_hierarchy]].

---

## The principle (binding)

A diagnostic of **severity ≥ Warning** (Warning / Error / Fatal) must be emitted
**unconditionally** — never suppressed by trace/verbosity. **Verbosity governs only
informational output** (Info / Debug / Trace). A severity-bearing message hidden behind a
verbosity level is a *silent failure*.

## Why now (the trigger, source-cited)

PGEN's in-house diagnostic mechanism conflates verbosity with severity:
- `TraceLevel` (`ast_pipeline/mod.rs`) is a **pure verbosity scale** — `Low=1, Medium=2,
  High=3, Debug=4`. There is **no Warning/Error/Fatal severity**.
- The central sink `trace_log` (`mod.rs`) gates *everything*: `if !trace_enabled(level)
  { return }`. The per-generator `trace()` (`stimuli_generator.rs:1248`) does the same
  (`if !self.config.trace_verbosity.allows(level) { return }`).
- Consequence: error text routed through `trace()` is **masked at low verbosity**. The
  exemplar — `Err(err) => self.trace(TraceLevel::High, "…error={}", err)` at
  `stimuli_generator.rs:4437` and `:4021` — hid the depth-exceeded error that was the
  dominant cause of the SV 888 stimuli residual for the entire `.7.2` campaign (see
  `SV-EXH-PROOF-7.4.3a-residual-failure-rootcause.md`).

The codebase already has always-on channels elsewhere (`eprintln!` / `log::warn!` /
`log::error!` in `main.rs`, `test_runner`, `ebnf_frontend`, `ast_shape_contract`, …), so
the defect is specific to the `ast_pipeline` custom trace mechanism.

---

## Leaves

- ID: `DIAG-SEVERITY.1`
  Status: `done` (`-0001`, 2026-06-02, pure-docs AUDIT — tools-first) — see "Audit" below
  Goal: `Enumerate every site in the WHOLE codebase where a Warning/Error/Fatal-severity
  message is emitted through a verbosity-gated path (the trace_log/trace mechanism + any
  verbosity-wrapped eprintln). Classify each: (A) value-lost (error only traced, not
  returned/handled), (B) visibility-masked (error returned as Result but its human
  surface is trace-gated), (C) genuinely informational/debug (correctly level-gated, no
  change). Produce the migration list.`
  Acceptance: `a source-cited site inventory + classification; no code change.`

- ID: `DIAG-SEVERITY.2`
  Status: `pending` (code + ADR — the severity mechanism)
  Goal: `Introduce a Severity dimension orthogonal to verbosity so Warning/Error/Fatal
  ALWAYS emit (bypass the verbosity gate, route to stderr), while Info/Debug/Trace stay
  verbosity-gated. Candidate: a Severity enum + a diag(severity, …) / diag_warn!/
  diag_error! API on the trace sink where severity ≥ Warning ignores trace_enabled; OR
  route severity-bearing sites to the existing log/eprintln always-on channel. Record the
  decision as an ADR in docs/decisions/. Parser-agnostic engine change.`
  Acceptance: `severity ≥ Warning emits at verbosity none; Info/Debug unchanged; unit
  test proving a warning/error is emitted with verbosity=none; lib+clippy green; ADR
  written; NO masking remains possible via the new API.`

- ID: `DIAG-SEVERITY.3`
  Status: `pending` (code — migrate the audited masked sites)
  Goal: `Reroute every (A)/(B) site from .1 to the always-on severity channel; keep
  (C) info/debug sites level-gated. Include the .7.4.3a exemplars (stimuli_generator.rs
  :4021, :4437) and the error-by-reason classification gap at :2432 (only timeouts are
  bucketed → add depth_exceeded / recursion_pressure / other reason buckets, surfaced in
  the gap-report summary). One module/site-group per sub-leaf, each measured.`
  Acceptance: `each masked error/warning now visible at verbosity none; error tallies
  carry a reason breakdown; lib+clippy green; behavioral proof (run at low verbosity, the
  message appears).`

- ID: `DIAG-SEVERITY.4`
  Status: `pending` (enforcement — cannot regress)
  Goal: `Add a guard so a Warning/Error/Fatal can never again be routed solely through a
  verbosity-gated path: a check-script / CI gate (mirror the MEMORY-ARCH E2/E4 pattern)
  + lint, e.g. flag any `Err(...) => …trace(`/level-gated emission carrying error/warn
  semantics. Wire into .githooks + CI.`
  Acceptance: `the guard fails on a reintroduced masked-severity site + passes clean on
  the migrated tree; runs in pre-commit + CI.`

- ID: `DIAG-SEVERITY.5`
  Status: `pending` (book lockstep + close)
  Goal: `Document the diagnostics severity model (verbosity vs severity; warnings+ always
  on) in the top-level mdBook (Developer Architecture / a Diagnostics section) + per the
  [[feedback_regex_book_live]] lockstep; close the tree.`
  Acceptance: `book updated + HTML rebuilt + mdbook_docs_gate green; tree closed.`

---

## Audit (DIAG-SEVERITY.1 findings — 2026-06-02, tools-first)

**Mechanism (the root defect):** `TraceLevel` carries no severity; `trace_log` (mod.rs)
and `trace()` (stimuli_generator.rs:1248) both early-return when verbosity < level. So
ANY message passed through them — including error text — is maskable.

**Sites found (initial inventory; `.3` confirms/extends per module):**
- `ast_pipeline/stimuli_generator.rs:4437` — **(B) visibility-masked.** `Err(err) =>
  self.trace(TraceLevel::High, "↰ exit generate_rule … error={}", err)`. The Err VALUE
  still propagates up the Result, but its human-readable surface is trace-gated → the
  depth-exceeded message vanished at the gate's low verbosity. **Exemplar.**
- `ast_pipeline/stimuli_generator.rs:4021` — **(B) visibility-masked.** Same `Err(err)
  => self.trace(…)` shape on another generation exit path.
- `ast_pipeline/stimuli_generator.rs:2432` — **error-classification gap (related).** The
  target-drive loop's `Err` arm tallies a generic `generation_errors` and sub-buckets
  ONLY the two timeout kinds; depth-exceeded (and every non-timeout error) lands in the
  generic count with **no reason** → invisible in the gap-report summary. Fix under `.3`.
- `ast_pipeline/semantic_runtime.rs:1740, 2047, 2145` — **(C) genuinely debug.**
  `if trace_enabled(TraceLevel::Debug) { … }` guards around debug breadcrumbs (predicate
  evaluation tracing). Correctly level-gated; no change (confirm in `.3`).
- `TraceLevel::` usage census: stimuli_generator.rs (54), mod.rs (13), semantic_runtime
  (3), ast_based_generator (1). `.3` walks each to confirm severity vs info.

**Classification summary:** the masking is concentrated in the `ast_pipeline` custom
trace mechanism; the rest of the codebase uses always-on `eprintln!`/`log`. The general
fix is `.2`'s Severity dimension (Warning+ bypasses the verbosity gate), then `.3`
migrates the (B) sites + closes the (A)-risk by construction.

---

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `DIAG-SEVERITY.1` | `done` (`-0001`) | Audit complete (above). |
| 1 | `DIAG-SEVERITY.2` | `pending` | The severity mechanism (Warning+ bypasses verbosity) + ADR. |
| 2 | `DIAG-SEVERITY.3` | `pending` | Migrate masked sites + error-by-reason buckets. |
| 3 | `DIAG-SEVERITY.4` | `pending` | Enforcement gate (cannot regress). |
| 4 | `DIAG-SEVERITY.5` | `pending` | Book lockstep + close. |

## Decisions

- `2026-06-02` (creation + principle): severity ≥ Warning is never verbosity-gated;
  trace verbosity is info-only. Whole-codebase correction owned here. The general
  mechanism (a Severity dimension) is the signoff fix, not per-site `eprintln!` patches —
  though migrated sites may route to the existing always-on channel.
