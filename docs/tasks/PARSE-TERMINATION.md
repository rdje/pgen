# PARSE-TERMINATION — the parser never hangs; bounded, near-linear time (parser-agnostic)

> Task tree. **Metadata** — Status: `active` (`.1` measurement DONE 2026-06-03 — CC 2020
> risk CONFIRMED empirically); Created: 2026-06-03; Roadmap lane: parser sign-off pillar
> **B** (of 4). Owns failure mode
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
### `.1` — MEASURE whether PGEN's stateful packrat is actually linear (tools-first) — DONE (2026-06-03)
Built a complexity-scaling probe: parsed SV inputs of size N ∈ {100..3200} via
`parseability_probe --parse systemverilog`, two families — **store** (N `typedef t; t v;`
pairs, exercising the store-gated type-identifier rule = the stateful path) and **width**
(N `wire w;`, stateless baseline). **RESULT (decisive, CC 2020 risk CONFIRMED):**
- **store (stateful):** 100→3200 = 32× input, 0.11 s → **34.8 s** = 316× → **~N^1.66**, and
  the per-doubling ratio *grows* monotonically (2.45 → 2.89 → 3.24 → 3.58 → 3.84 →≈4) —
  **trending QUADRATIC**.
- **width (stateless):** 0.04 s → 1.70 s = 42× → **~N^1.08 = linear** (packrat works as
  advertised when there's no state).
**So PGEN's stateful (semantic-store) packrat is NOT linear** — exactly Chida & Kawakoya
CC 2020. Likely explains historical uvm_pkg slowness (thousands of type decls/uses → the
super-linear store path). Not a hang (all completed), but a real super-linear risk at scale.
No code change (measurement only). Inputs/timings under `rust/target/w74/lin/` (gitignored).
KM card [[stateful-packrat-not-linear]] updated with the empirical confirmation. → justifies
`.3` (conditional memoization) as the fix.

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
`.1` DONE — measurement CONFIRMED super-linear (≈quadratic-trending) stateful parsing.
Next: `.3` (memo-soundness audit → **conditional memoization**, CC 2020's fix — restores
linearity for store-gated rules; a parser-agnostic engine change, explicit auth + strict
scope) is now evidence-justified; `.5` (complexity-regression gate) can lock the curve in CI
so a regression can't reappear. Fix path is grounded in `.1`'s facts, not a guess.
