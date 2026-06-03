# PARSE-TERMINATION — the parser never hangs; bounded, near-linear time (parser-agnostic)

> Task tree. **Metadata** — Status: `proposed` (literature-grounded; awaiting first leaf);
> Created: 2026-06-03; Roadmap lane: parser sign-off pillar **B** (of 4). Owns failure mode
> **(1.b) hang** — non-termination or super-linear blow-up (catastrophic backtracking) on
> valid input.
>
> Director-commissioned 2026-06-03 (one tree per failure mode; research-first). Parser-
> AGNOSTIC ([[feedback_ast_pipeline_parser_agnostic]]). Disciplines:
> [[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_tools_first_no_guessing]],
> [[feedback_prefer_grammar_leave_engine_alone]] (engine changes are parser-agnostic
> last-resort), [[feedback_severity_never_gated_by_verbosity]] (a watchdog error is a
> severity, always emitted).

## The principle (binding)
A PGEN parser shall **terminate in bounded (near-linear) time on every input**. A hang is a
correctness defect, not "slow." "Solve for good" = make non-termination **impossible by
construction** (well-formedness + a sound complexity guarantee) **plus** a runtime watchdog
that converts any residual pathological case into a *classified severity error*, never a
silent hang.

## ⚠️ The sweep's key warning (this tree's reason to exist)
**Packrat ≠ linear for PGEN.** Ford's linear-time guarantee (*Packrat Parsing*, ICFP 2002)
**assumes a stateless PEG**. PGEN's **semantic store makes it a *stateful* PEG**, and
**Chida & Kawakoya, *Is Stateful Packrat Parsing Really Linear in Practice?* (CC 2020)**
proves real-world stateful grammars can go **exponential**. So we may **NOT assume** PGEN is
hang-free — this is a live, previously-unexamined risk. See [[stateful-packrat-not-linear]].
**That same paper hands us the fix** (adopt, don't invent): a *PEG with variable bindings* +
**stateful packrat with conditional memoization** (memoize only the state relevant to global
-state use) — **260×/217×** time/space win on pathological inputs.

## Literature grounding (citations + worked mapping)
- ✅ **Ford, PEG well-formedness (POPL 2004 §3.6)** — a well-formed PEG cannot loop forever;
  static detection of non-terminating rules. = PGEN's existing **PARSE-SOTA A1**.
- ✅ **Warth, Douglass, Millstein (PEPM 2008)** — packrat + left recursion. = PGEN's
  `pre_lr_elim` + `mutual_recursion_handler`.
- ⚠️♻️ **Chida & Kawakoya (CC 2020)** — stateful non-linearity + conditional-memoization fix
  (above). Sharpens PGEN's already-backlogged **PARSE-SOTA "memo-soundness audit"** (Tier B).
- ✅ **Mizushima et al. (2010)** — cut operators bound backtracking.
- **Worked mapping:** PGEN already has A1 (static non-termination) + memoization + LR
  handling — two of three legs. The gap is the **complexity *guarantee*** under PGEN's
  statefulness, which CC 2020 says we cannot assume.

## Leaves
### `.1` — MEASURE whether PGEN's stateful packrat is actually linear (tools-first) — PENDING (next)
Per [[feedback_why_and_where_before_solution]]: build a **complexity-scaling probe** (parse
-time vs input-size on a scaling corpus per family) and look for super-linear growth BEFORE
any fix. This either clears the CC 2020 risk empirically or pins a concrete pathological
case to fix. No code change beyond the probe.

### `.2` — static no-hang surface: confirm + extend A1 — PENDING
Confirm A1 well-formedness covers every family; extend to *loop-without-consuming* detection
(a rule that can match empty inside `*`/`+`).

### `.3` — memo-soundness audit; adopt conditional memoization if needed — PENDING
Prove memoization delivers the intended complexity. If `.1` shows statefulness breaks it,
adopt CC 2020's **conditional memoization** (parser-agnostic engine change, explicit auth +
strict scope per the engine-change discipline).

### `.4` — runtime step-budget watchdog (hang → classified severity error) — PENDING
A hard per-parse step/time budget that converts a would-be hang into a `pgen_error!`-class
**severity** diagnostic (always emitted, never verbosity-gated — DIAG-SEVERITY), with the
reason classified. Bounded by construction; additive ⇒ zero regression.

### `.5` — complexity-regression gate (CI) — PENDING
Assert sub-quadratic parse-time slope on the scaling corpus; catches a hang's *precursor*
(super-linear) before it becomes a hang.

## Frontier
`.1` (measure linearity — resolves the CC 2020 risk with facts). The fix path (`.3`/`.4`)
follows the measurement, not a guess.
