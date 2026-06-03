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

### `.2` — static no-hang surface: nullable-repetition detector — DONE (PGEN-PARSE-TERMINATION-0002, 2026-06-03)
Extended `grammar_wellformedness.rs` with `detect_nullable_repetition` — flags an UNBOUNDED
quantifier (`*`/`+`/`{N,}`, i.e. `max == None`) over a NULLABLE body (the "loop without
consuming" hazard, Ford PEG well-formedness POPL 2004 §3.6). Reuses the existing
`compute_nullable`/`node_nullable` fixpoint; deterministic, parser-agnostic. New
`WellformednessIssue::NullableRepetition { rule, node_path }` + wired into `--lint-grammar`
as a WARNING (runtime is zero-length-guarded, so it's not an actual hang — but the grammar is
ill-formed). Unit-tested (positive + no-false-positive on non-nullable/bounded); lib 588/588;
source-strict clippy 0.
**RAN on shipped grammars: regex/ebnf/vhdl/rtl_frontend = 0; SystemVerilog = 7 real findings:**
`bins_or_empty` (root/o0), `bins_or_options` (root/o1/s5 — note: also a deeply-factored
slow-witness rule from `.7.4`), `module_path_concatenation` (root), `rs_code_block` (root),
`rs_production_list_sv_2017`/`_2023` (root/o0/s1), `select_condition` (root/s4/q/s1). These
are genuine ill-formed sites (runtime-guarded, low-urgency) → the 7 grammar FIXES are
follow-up targeted leaves (`.2.1`-`.2.7`, each: analyze intended quantifier/body, fix, regen
+ verify corpus/shape-contracts — NOT done here; the detector is the `.2` deliverable).

### `.3` — root-cause + fix DESIGN (DONE, PGEN-PARSE-TERMINATION-0003); implementation = `.3.1`
Director signed off the engine change. WHY+WHERE-first investigation (tools: code read +
macOS `sample` profile of the store_3200 parse) **PINNED the root cause and CORRECTED the
leaf's premise** — it is **NOT** CC 2020 conditional memoization:
- **ROOT CAUSE (profiled, decisive):** `with_semantic_runtime_rule_transaction` (codegen
  `ast_based_generator.rs:1281-1283`) does `std::mem::take(&mut self.semantic_runtime_state)`
  then `= original.clone()` — a **full clone of the entire `SemanticRuntimeState`** (facts
  BTreeMap/HashMap, scopes, strings, vecs) on EVERY rule transaction, restored by `= original`
  on failure. O(state) per call × O(rule-calls) = **O(N²)**. The profile's hot frames are
  `SemanticRuntimeState::clone` (clone_subtree / String::clone / HashMap::clone / to_vec) +
  `drop_in_place<SemanticRuntimeState>`, all under `with_semantic_runtime_rule_transaction`.
  (NOT the memo — keyed (rule,pos), never cleared; NOT has_fact — indexed `by_kind`, O(1);
  the width baseline is linear because it runs no transactions over a growing store.)
- **FIX (efficient snapshot/restore — Laurent & Mens SLE 2016):** replace the take+clone
  snapshot with `self.semantic_runtime_state.checkpoint()` (O(1)) and the err-restore
  `= original` with `self.semantic_runtime_state.rollback_to_named(cp, Some(rule_name))`
  (O(changes)). `rollback_to_named` is VERIFIED COMPLETE (truncates facts + the by_kind
  fact_index, truncates scope_arena, restores active_chain, **un-closes** scopes, rebuilds
  `scopes`) and is ALREADY used + proven in the try_parse branch path (SV corpus 14/14). Also
  translate the in-IIFE `original_semantic_runtime_state.facts().len()` (`:1385`) →
  `cp.fact_len`; rule_context push/pop is unaffected (checkpoint/rollback don't touch it).
- **EXPECTED IMPACT:** per-transaction O(state) → O(changes) ⇒ store-gated parsing
  N^1.66 → ~N^1.0 (linear, matching the width baseline); fixes the super-linearity + likely
  the historical uvm_pkg slowness.

### `.3.1` — implement the checkpoint/rollback swap — PENDING (NOT a clean swap — subtlety below)
Apply the fix in `with_semantic_runtime_rule_transaction` (codegen `ast_based_generator.rs`).
`generated/` is GITIGNORED (not committed) → the commit is the codegen `.rs` only; regen is
local verification. The 4 sites: snapshot (`:1281-1283`), entry-fact-len read (`:1385`),
failure-restore (`:1552`), and the codegen self-test assertion (`:7622`,
"let original_semantic_runtime_state" → update to assert checkpoint/rollback).
⚠️ **DISCOVERED SUBTLETY (2026-06-03, found while scoping the edit — NOT a 3-line swap):** the
success path does `std::mem::take(&mut self.semantic_runtime_state)` mid-function (`:1389`)
for the commit/delta machinery, then restores it (`:1546`). The original full-clone restore
(`self.state = original`) is robust to a failure BETWEEN the take and the put-back. A naive
`rollback_to(checkpoint)` would run on an EMPTIED `self.state` on that path → cannot restore
pre-checkpoint facts → **lost-facts correctness hole**. So `.3.1` must EITHER restructure the
transaction/commit machinery to operate on `self.state` in place (no mid-`take`) OR
restore-the-taken-state-before-rollback on the failure path. This is a careful refactor of the
transaction machinery, not a swap → a dedicated, fully-tested focused slice. VERIFY: `.1`
linearity probe (store → ~N^1.0), SV external corpus 14/14, lib (no-features +
generated_parsers), shape-contracts, determinism, AND a targeted test for the
failure-between-take-and-putback path. KM [[stateful-packrat-not-linear]].

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
