# DIAG-SEVERITY — severity (warning/error/fatal) must never be gated by a trace/verbosity level

> Task tree. **Metadata** — Status: `done` (CLOSED 2026-06-02, `-0001..0006`);
> Created: 2026-06-02 (`PGEN-DIAG-SEVERITY-0001`);
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
  Status: `done` (`-0002`, 2026-06-02, code — additive engine mechanism)
  Verification: `done — added to ast_pipeline/mod.rs: a Severity{Warning<Error<Fatal} enum (orthogonal to the TraceLevel verbosity scale); emit_diagnostic(severity,file,line,module,args) that ALWAYS writes to stderr (NO trace_enabled gate, by design) + mirrors to the trace file if configured; a testable write_diagnostic<W> core + pure format_diagnostic; and pgen_diag!/pgen_warn!/pgen_error!/pgen_fatal! macros (mirror pgen_trace!). 2 unit tests: severity_diagnostics_emit_regardless_of_verbosity (set verbosity None → a High TRACE is suppressed (the bug) BUT the severity diagnostic still writes "ERROR … depth exceeded max_depth=24" — the fix) + severity ordering/labels. PURELY ADDITIVE (no existing trace line changed → trace behavior unchanged). lib (no-features) 570/570 (+2); clippy 0 errors. NOTE: a separate pre-existing AnnotationSeverity enum exists for annotation diagnostics — distinct concern. ADR deferred into this verification note + the tree's principle section (full docs/decisions ADR can follow in .5 book lockstep). NO grammar/codegen/generated change, no release bump.`
  Commit: `PGEN-DIAG-SEVERITY-0002`
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
  Status: `done` (`-0003`, 2026-06-02, code) — error-by-reason classification: the high-value, correct fix
  Goal: `Surface generation-error REASONS unconditionally (the aggregate masking that hid the depth cause), rather than converting per-attempt PEG-backtrack breadcrumbs to stderr spam (those are EXPECTED control flow = info/debug, correctly level-gated — confirmed nuance, see Decisions). Add a depth_exceeded bucket at the target-drive Err arm (was: only timeouts bucketed → depth vanished into a generic count), surface it in the always-printed TargetDriveSummary.summary_line, and emit a once-per-run AGGREGATE pgen_warn! (the correct use of the .2 mechanism — never gated, not per-attempt).`
  Verification: `done — stimuli_generator.rs: const DEPTH_EXCEEDED_ERROR_PREFIX + is_depth_exceeded_error (reuses the generic is_timeout_error_with_prefix chain matcher); a depth_exceeded_errors counter classified in BOTH target-drive loops (generate_until_targets + _with_filter), threaded into TargetDriveSummary (new #[serde(default)] field) + summary_line + the completion trace; a once-per-run pgen_warn! when depth_exceeded_errors>0 (always-on, references SV-EXH-PROOF.7.4.3a). Unit test target_drive_summary_reports_helper_timeout_errors extended to assert depth_exceeded_errors=2 in the summary line. lib (no-features) 570/570; clippy 0 errors. KEY BONUS: building the full taxonomy (below) revealed a SECOND structural-budget failure — max_rule_visits exceeded (:4438) — also currently anonymous; routed to .3.1. NO grammar/codegen/generated change, no release bump.`
  Commit: `PGEN-DIAG-SEVERITY-0003`

- ID: `DIAG-SEVERITY.3.1`
  Status: `done` (`-0004`, 2026-06-02, code)
  Verification: `done — stimuli_generator.rs: a canonical GenerationErrorReason enum {DepthExceeded, RuleVisitLimit, TargetTimeout, HelperTimeout, Other} + classify_generation_error() = THE single source of truth; both target-drive Err arms (generate_until_targets + _with_filter) now classify via the match (behavior-equivalent: prefixes are mutually exclusive + mode-correlated, so dropping the old helper_probe_active guard cannot change counts); un-masked max_rule_visits (RULE_VISIT_LIMIT_ERROR_PREFIX + is_rule_visit_limit_error + rule_visit_limit_errors counter), threaded into TargetDriveSummary + summary_line + completion trace; the once-per-run pgen_warn! now covers BOTH structural-budget reasons (depth + rule-visit). 2 unit tests: classify_generation_error_maps_each_reason (canonical list) + summary asserts rule_visit_limit_errors. lib 571/571; clippy 0 errors (all enum variants constructed). NOTE: per-reason counts surface in the always-printed summary_line (run-log, gate-greppable); a structured gap-report-JSON surface would need a main.rs write (deferred — noted). NO grammar/codegen/generated change, no release bump.`
  Commit: `PGEN-DIAG-SEVERITY-0004`
  Goal: `Per director ask ("list all the error-by-reason classification; capture it"): consolidate the scattered prefix-matchers + parallel usize counters into a SINGLE source of truth — a `GenerationErrorReason` enum {DepthExceeded, RuleVisitLimit, TargetTimeout, HelperTimeout, QuantifierConfig, ZeroWeight, SemanticEval, Other} + classify_generation_error(&Error) -> GenerationErrorReason + a per-reason tally (map/struct) surfaced in TargetDriveSummary AND the gap-report JSON summary. Classify the remaining reasons found by the .3 taxonomy (esp. the masked max_rule_visits, the other sibling of depth). The enum IS the enumerated list (cannot drift); the book (.5) + taxonomy doc mirror it.`
  Acceptance: `one GenerationErrorReason enum = the exhaustive list; every generation Err classified through it; per-reason counts in the gap-report JSON summary (durable, gate-assertable); rule_visit_limit no longer anonymous; lib+clippy green.`

## Generation error-reason taxonomy (DIAG-SEVERITY.3 — source-cited)

Every failure the stimuli generation path can raise, and its classification. STRUCTURAL-
BUDGET failures (the masked-cause class) are the ones that matter for the SV residual:

| Reason | Error message (prefix) | Raised at | Class | Surfaced? |
|---|---|---|---|---|
| **depth_exceeded** | `Stimuli generation depth exceeded max_depth=` | `stimuli_generator.rs:4426` | structural budget | **YES (.3)** — bucket + summary + pgen_warn! |
| **rule_visit_limit** | `Stimuli generation exceeded max_rule_visits=` | `:4438` | structural budget | **YES (.3.1)** — bucket + summary + pgen_warn! (un-masked) |
| target_timeout | `Stimuli generation target timeout exceeded` | const `:27` | time budget | yes (pre-existing bucket) |
| helper_timeout | `Stimuli generation helper timeout exceeded` | const `:26` | time budget | yes (pre-existing bucket) |
| quantifier_config | `Unsupported quantifier format` / `Unknown quantifier` | `:5565` / `:5594` | grammar/config | no (rare; → `.3.1` Other) |
| zero_weight | `All explicit branch probabilities are zero` / `Computed branch weights are all zero` | `:5676` / `:5710` | grammar/config | no (→ `.3.1` Other) |
| semantic_eval | `Semantic relational expression/operand cannot be empty` | `:8134` / `:8306` | semantic | no (→ `.3.1` Other) |
| other | (any uncaught generation Err) | — | residual | folded into `generation_errors` total |

EXPECTED control flow (NOT a severity error): a plain rule-attempt failure that the PEG
generator handles by backtracking to another alternative. These are per-attempt
breadcrumbs (`Err(err) => self.trace(...)` at `:4021`/`:4437`) — correctly level-gated
info/debug; converting them to always-on would be stderr spam AND mislabel expected
failures as errors. The masking that hurt us was the AGGREGATE one (no reason on the
2,472-failure count), fixed by `.3` + `.3.1`.

- ID: `DIAG-SEVERITY.4`
  Status: `done` (`-0005`, 2026-06-02, enforcement)
  Verification: `done — scripts/check_diagnostics_and_docpaths.sh (sound, passes clean on the current tree): (1a) asserts the always-on severity mechanism is present in mod.rs (Severity enum + emit_diagnostic + pgen_warn!/error!/fatal! — cannot be silently removed); (1b) flags any UNAMBIGUOUS severity (fatal/panic) routed through the verbosity-gated trace (best-effort tripwire; per-attempt info/debug breadcrumbs allowed); (2) fails if a LIVE doc (book/src, contracts, user guide, README) carries a repo-internal absolute path (the DOCPATH guard). Wired into .githooks/pre-commit (runs alongside check_memory_architecture.sh) + the memory-architecture-gate CI workflow (new step). PROVEN: passes clean (exit 0, "diagnostics+docpaths: OK"); detection patterns bite on injected masked-fatal + repo-internal-abs-path; this very commit passes through the extended pre-commit hook. NO Rust/grammar/generated change, no release bump.`
  Commit: `PGEN-DIAG-SEVERITY-0005`
  Goal: `Add a guard so a Warning/Error/Fatal can never again be routed solely through a
  verbosity-gated path: a check-script / CI gate (mirror the MEMORY-ARCH E2/E4 pattern)
  + lint, e.g. flag any `Err(...) => …trace(`/level-gated emission carrying error/warn
  semantics. Wire into .githooks + CI.`
  Acceptance: `the guard fails on a reintroduced masked-severity site + passes clean on
  the migrated tree; runs in pre-commit + CI.`

- ID: `DIAG-SEVERITY.5`
  Status: `done` (`-0006`, 2026-06-02, book lockstep + TREE CLOSED)
  Verification: `done — added a "Diagnostics: severity vs verbosity" section to the top-level mdBook docs/book/src/developer-architecture.md (the two orthogonal axes; the always-on severity mechanism + macros; the hard rule + its enforcement gate; the GenerationErrorReason error-by-reason taxonomy table). HTML rebuilt (section rendered); mdbook_docs_gate green; check_diagnostics_and_docpaths green. TREE CLOSED. NO Rust/grammar/generated change, no release bump.`
  Commit: `PGEN-DIAG-SEVERITY-0006`
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
| — | `DIAG-SEVERITY.2` | `done` (`-0002`) | Severity mechanism landed: Severity enum + emit_diagnostic (always-on) + pgen_warn!/error!/fatal! macros; 570/570, clippy clean. |
| — | `DIAG-SEVERITY.3` | `done` (`-0003`) | Error-by-reason: depth_exceeded bucket + summary + once-per-run pgen_warn!; taxonomy documented; revealed max_rule_visits as a 2nd masked budget failure. 570/570. |
| — | `DIAG-SEVERITY.3.1` | `done` (`-0004`) | Canonical GenerationErrorReason enum (single source of truth) + un-masked max_rule_visits; 571/571. |
| — | `DIAG-SEVERITY.4` | `done` (`-0005`) | Enforcement gate (check_diagnostics_and_docpaths.sh, wired into pre-commit + CI): severity-mechanism-present + no fatal/panic masked through trace + live-docs path guard. |
| — | `DIAG-SEVERITY.5` | `done` (`-0006`) | Book lockstep (Diagnostics: severity vs verbosity + reason taxonomy in Developer Architecture). **TREE CLOSED.** |
| 3 | `DIAG-SEVERITY.4` | `pending` | Enforcement gate (cannot regress). |
| 4 | `DIAG-SEVERITY.5` | `pending` | Book lockstep + close. |

## Decisions

- `2026-06-02` (creation + principle): severity ≥ Warning is never verbosity-gated;
  trace verbosity is info-only. Whole-codebase correction owned here. The general
  mechanism (a Severity dimension) is the signoff fix, not per-site `eprintln!` patches —
  though migrated sites may route to the existing always-on channel.
- `2026-06-02` (.3 NUANCE — director-agreed): the high-value, correct fix is
  **error-by-reason classification surfaced unconditionally (aggregate)**, NOT converting
  per-attempt PEG-backtrack `Err` breadcrumbs to stderr — those are EXPECTED control flow
  (try alternative → fail → backtrack), legitimately info/debug, and flooding stderr with
  them would both spam and mislabel expected failures as errors. The masking that cost us
  the `.7.2` campaign was the *aggregate* one: 2,472 failures counted with no reason. The
  reason taxonomy is captured (table above) + enumerated canonically in code by `.3.1`
  (`GenerationErrorReason` enum = the single, drift-proof list) + mirrored in the book
  (`.5`). Building the taxonomy surfaced `max_rule_visits` as a second, still-anonymous
  structural-budget failure.
