# SV-EXH-PROOF: Main-SystemVerilog formally-exhaustive machine-checkable closure proof

## Metadata

- Tree ID: `SV-EXH-PROOF`
- Status: `active`
- Roadmap lane: parser-family exhaustive-proof normalization (the last open parser-family proof debt)
- Created: `2026-05-17`
- Last updated: `2026-05-17` (`.1` measured baseline complete — tree re-planned to 6 leaves; see Decisions/Verification)
- Owner: repo-local workflow

## Goal

Re-earn `Done` for the `systemverilog` main-parser family by closing
the single primary unmet closure criterion that SV's own
machine-checkable contract names (`formal_exhaustive_closure_surface_green`,
blocked on the missing **`external_corpus_backed_proof_surface`**) —
**honestly**: with a derived, checked-in, deterministic proof surface
over a genuinely-green external-corpus parse state, not a hard-coded
literal over a failing surface.

## Non-Goals

- **NOT** porting the `systemverilog_preprocessor`
  reachability/syntax/zero-plausible-gap trio (falsified
  `PGEN-SV-EXH-PROOF-0001`; SV-main static syntax-closure is already
  closed — confirmed healthy by the `.1` baseline, Finding B).
- Not the broader Phase-S build-out (Liberty/SDC crates, rtl_frontend
  parity).

## Acceptance Criteria

- The preprocessor syntax-closure regression (baseline Finding A) is
  remediated; `sv_preprocessor_syntax_closure_gate` +
  `sv_parser_family_status_gate` + `sv_formal_exhaustive_closure_gate`
  run green.
- The external-corpus parse surface is genuinely green (every declared
  case parses full in both profiles) **or** every residual parse-fail
  is explicitly dispositioned in a checked-in per-case contract with
  honest rationale — no false closure claims.
- `sv_formal_exhaustive_closure_gate.sh` derives
  `external_corpus_backed_proof_surface_present` from that checked-in
  contract (the hard-coded `true`, baseline Finding D, is removed);
  `systemverilog_formal_exhaustive_closure_contract.json` flipped
  "surface missing" → "surface present + proof path";
  `sv_parser_family_status_gate` closure criteria all satisfied;
  `sota_exit_gate` + `sv_combined_telemetry_contract_gate` parity.
- The two LIVE rows flip to `Done` with the machine-checkable surface
  (not narrative) as evidence; SV book + integration contract
  same-commit lockstep; no regression to existing SV gates; full
  COMMIT.md lockstep per leaf.

## Task Tree

- ID: `SV-EXH-PROOF`
  Status: `active`
  Goal: `Close SV's sole unmet closure criterion (formal_exhaustive_closure_surface_green) with a derived honest external_corpus_backed_proof_surface over a genuinely-green corpus parse state; re-earn Done for the SV main-parser family.`
  Children: `SV-EXH-PROOF.1`, `SV-EXH-PROOF.2` (parent: `.2.1`, `.2.2`), `SV-EXH-PROOF.3` … `SV-EXH-PROOF.6`

- ID: `SV-EXH-PROOF.1`
  Status: `done`
  Goal: `Deterministic measured baseline of the four existing gates + scope lock + mandatory LIVE-tracker drift correction (docs-only).`
  Acceptance: `Checked-in measured baseline (docs/SV_EXH_PROOF_BASELINE.md) with the true numbers + 4 findings (A preprocessor syntax-closure REGRESSED/blocker; B SV-main static-closure healthy; C external-corpus 0/14 not 10/14; D hard-coded surface_present=true); LIVE drift corrected same-commit; tree re-planned.`
  Verification: `done — see Verification Log 2026-05-17 (.1)`
  Commit: `PGEN-SV-EXH-PROOF-0002`

- ID: `SV-EXH-PROOF.2`
  Status: `active` (parent)
  Goal: `Remediate the preprocessor regression FAMILY left by the POST-SV-AUDIT.2.1 + INLINE-ALT-FIX.1 grammar edits (a cascade of un-lockstepped downstream preprocessor proof-stack expectations; the grammar edits are legitimate correctness fixes — NOT to be reverted; re-lockstep the proof surfaces). Restore sv_preprocessor_zero_plausible_gap_proof_gate + sv_parser_family_status_gate to green.`
  Acceptance: `sv_preprocessor_zero_plausible_gap_proof_gate + sv_parser_family_status_gate green; every re-lockstep is evidence-grounded (underlying behavior genuinely satisfied, not weakened); contract/script changes leaf-owned + documented.`
  Children: `SV-EXH-PROOF.2.1`, `SV-EXH-PROOF.2.2`, `SV-EXH-PROOF.2.3`

- ID: `SV-EXH-PROOF.2.1`
  Status: `done`
  Goal: `(A1) Re-baseline systemverilog_preprocessor_syntax_closure_contract.json max_unreachable_branches 3 -> 13 (legitimate added named-rule structure; genuine static-unreachable surface is still ONLY the benign trivia pocket per unreachable_*_debt, version 1->2, evidence in description). (A2) Re-target the stale sv_preprocessor_quality_gate.sh:723 assertion from the removed pp_if_branch::root/s0 inline path to the post-SVPP-0001 lifted pp_if_keyword::root branch group (intent preserved; underlying coverage [7,6] genuinely satisfies — not weakened).`
  Acceptance: `sv_preprocessor_syntax_closure_gate green (verified: status pass, unreachable_branches=13, unreachable_rules=1, reachable_rules=72); A2 retarget verified against real coverage (pp_if_keyword::root success_counts=[7,6]).`
  Verification: `done — see Verification Log 2026-05-17 (.2.1)`
  Commit: `PGEN-SV-EXH-PROOF-0003`

- ID: `SV-EXH-PROOF.2.2`
  Status: `done`
  Goal: `(A3') Remediate the "reachable-branch universe drifted across stages: stage0=10 stage1=0" sv_preprocessor_aggregate_contract_gate failure. Root cause (documented): summary.reachable_branches is a burn-down DEBT metric (stimuli_generator.rs:1589 skips deficit==0), NOT a static universe; the Cat-A macro_default_atom factoring made stage0 leave 10 uncovered that stage1 covers (covered_branches 37->47) — desirable burn-down wrongly flagged by a byte-equality assertion. The true static universe (total_rules=73/total_branches=50/reachable_rules=72) is stage-stable everywhere.`
  Acceptance: `Mis-specified reachable_* cross-stage EQUALITY replaced by (a) total_* stage-equality (the true static universe — strengthened, holds) + (b) reachable_* non-increasing burn-down (genuine no-regression guarantee, catches real debt-growth). sv_preprocessor_aggregate_contract_gate no longer fails the drift check; sv_preprocessor_zero_plausible_gap_proof_gate runs to completion (verified MAKE_RC=0, unreachable surface confined to trivia pocket: observed=["trivia"] ⊆ allowed). Not weakened/masked.`
  Verification: `done — see Verification Log 2026-05-17 (.2.2)`
  Commit: `PGEN-SV-EXH-PROOF-0004`

- ID: `SV-EXH-PROOF.2.3`
  Status: `pending`
  Goal: `(A4) The preprocessor zero-plausible-gap proof verdict is red on "Aggregate preconditions regressed: parseability_parser_rejections_total=3" (hard ==0 requirement, sv_preprocessor_zero_plausible_gap_proof_gate.sh:234) — the closed-loop generates 3 directive stimuli the preprocessor grammar self-rejects ("Parser did not consume full input"; shrunk repro for all 3 = a bare backtick "\`", which the grammar correctly rejects: non_directive_text excludes "\`" and no rule accepts a lone backtick → a generator⊋parser asymmetry). **Premise correction (PGEN-SV-EXH-PROOF-0005): NOT campaign-caused.** The exact diffs of a5da52f4 (SVPP-0001) and 7228231b (POST-SV-AUDIT.2.1) are generatively INERT — a5da52f4 lifts (kw_ifdef|kw_ifndef) into the structurally-equivalent named rule pp_if_keyword (identical generated/parsed language); 7228231b changes ONLY the macro_formals -> annotation ({first,rest} -> [$2,$3::2*]), the production is unchanged. The earlier "genuine campaign-caused round-trip regression / was 0 at preproc Done 2026-04-01" was an UNVERIFIED inference now falsified. Real task: bisect what actually moved parser_rejections 0->3 (non-grammar pipeline: stimuli_generator/codegen/parser-gen evolution this session, OR a pre-existing seed-sensitive generator⊋parser asymmetry that the 2026-04-01 seed/count didn't surface), then honest fix (grammar-harden so closed-loop round-trips, or fix the generator emitting a dangling backtick — NOT loosen the ==0 precondition to mask a real self-rejection).`
  Acceptance: `parseability_parser_rejections_total=0 in the preprocessor closed-loop; sv_preprocessor_zero_plausible_gap_proof_gate verdict GREEN (helper_only_unreachable_surface_green=true, zero_plausible_grammar_level_gap_proof_surface=true); root cause of the 0->3 move identified + honestly fixed (no tolerance loosened).`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.3`
  Status: `pending`
  Goal: `SV-main grammar hardening: drive the external-corpus parse surface 0/14 -> green AND fix the SV-main closed-loop replay-shadow rejections (baseline Finding A3: sv_parser_aggregate_contract_gate "replay-shadow totals internally inconsistent" — SV-main rejects valid SV: escaped identifiers \\foo, export *::*;, package-body constructs; same root class as Finding C). Triage the regressing commit(s); close per-corpus/per-defect parse-fails (uvm, scr1, friscv, veer_el2) + the closed-loop replay-shadow rejections across sv_2017+sv_2023. Multi-slice; each grammar fix is its own sub-leaf, probe-verified + lockstepped.`
  Acceptance: `parse_pass_total == cases_executed (or every residual explicitly dispositioned for .4); each fix probe-verified; no AST-shape / aggregate / stimuli regression; per-fix contract+book lockstep.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.4`
  Status: `pending`
  Goal: `Build the real external_corpus_backed_proof_surface: a checked-in deterministic per-case disposition contract + generator; sv_external_corpus_triage_gate-derived surface_present (replace the hard-coded true, baseline Finding D); every parse-fail closed or honestly justified-bounded with rationale.`
  Acceptance: `Sidecar + contract checked in; deterministic + repeatable; surface_present is derived, not literal; no false closure.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.5`
  Status: `pending`
  Goal: `Wire the surface in: flip systemverilog_formal_exhaustive_closure_contract.json "surface missing" -> "surface present + proof path"; sv_formal_exhaustive_closure_gate.sh consumes the derived surface; sv_parser_family_status_gate + sota_exit_gate + sv_combined_telemetry_contract_gate parity.`
  Acceptance: `Formal-exhaustive gate green requiring the real derived surface; family-status closure criteria all satisfied; telemetry parity machine-checked; no regression.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.6`
  Status: `pending`
  Goal: `Flip the two LIVE rows -> Done with the machine-checkable surface as evidence; SV per-parser book + integration contract same-commit lockstep; full closeout + tree close.`
  Acceptance: `LIVE "systemverilog main parser" + "Parser-family exhaustive proof normalization" rows Done with evidence; book + contract lockstepped; tree closed; promoted to Completed in TASK_TREE.md.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-EXH-PROOF.2.3` | `pending` | `.2.1` (A1+A2) + `.2.2` (A3' universe-drift mis-spec) done+verified. `.2.2` let the gate complete, surfacing the next cascade layer: the preprocessor zero-gap proof's `parseability_parser_rejections_total=3` — 3 closed-loop directive stimuli the refactored grammar self-rejects. Genuine campaign-caused round-trip regression; blocks the preprocessor zero-gap proof verdict. |
| 2 | `SV-EXH-PROOF.3` | `pending` | SV-main grammar hardening 0/14 → green + the SV-main closed-loop replay-shadow rejections (Finding A3); the large multi-slice body; needs `.2` so the umbrella can validate progress. |
| 3 | `SV-EXH-PROOF.4` | `pending` | Build the derived external-corpus-backed proof surface (needs `.3`'s genuinely-green/dispositioned state). |
| 4 | `SV-EXH-PROOF.5` | `pending` | Wire it into the contract/gate/family-status/telemetry (needs `.4`). |
| 5 | `SV-EXH-PROOF.6` | `pending` | LIVE `Done` flip + book/contract lockstep + closeout (needs `.5` green). |

## Decisions

- `2026-05-17`: User selected this workstream from the
  post-`POST-SV-AUDIT` strategic fork.
- `2026-05-17` (`PGEN-SV-EXH-PROOF-0001`): preprocessor-trio-port
  hypothesis falsified; SV-main static syntax-closure already present;
  sole gap = the missing `external_corpus_backed_proof_surface`.
- `2026-05-17` (`PGEN-SV-EXH-PROOF-0002`, **`.1` measured baseline**):
  ground-truth measurement (`docs/SV_EXH_PROOF_BASELINE.md`) produced
  four findings: **(A)** `sv_preprocessor_syntax_closure_gate` is
  REGRESSED on `main` (`unreachable_branches=13 > 3`) — a real
  lockstep defect from this session's POST-SV-AUDIT.2.1 +
  INLINE-ALT-FIX.1 preprocessor grammar edits (contract never
  re-baselined); blocks the SV family-status/formal-exhaustive
  umbrella. **(B)** SV-main static syntax-closure is healthy/pass
  (re-scope validated). **(C)** external-corpus parse surface is
  `0/14` genuine grammar rejections, NOT the `10/14` the LIVE tracker
  claimed (proven-false drift — corrected same-commit). **(D)**
  `sv_formal_exhaustive_closure_gate.sh:245` hard-codes
  `surface_present=true` (unproven literal). Tree re-planned to 6
  leaves: prerequisite preprocessor remediation (`.2`) → SV grammar
  hardening `0/14→green` (`.3`) → derived proof surface (`.4`) →
  wiring (`.5`) → LIVE `Done` + lockstep (`.6`). PNT-eligible: the
  workstream is user-fixed; decomposition + honest reporting of
  discovered regressions is the implementer's call (ground truth
  unambiguous; the discovered preprocessor regression is recorded as a
  tracked defect, not silently fixed).
- `2026-05-17` (`PGEN-SV-EXH-PROOF-0003`, **`.2.1`**): the
  preprocessor "regression" is a **cascade** of un-lockstepped
  downstream proof-stack expectations from the same (legitimate,
  correctness-improving — NOT to be reverted) POST-SV-AUDIT.2.1 /
  INLINE-ALT-FIX.1 grammar edits. Fixed+verified at their gate level:
  **A1** syntax-closure contract re-baseline (`max_unreachable_branches`
  3→13, version 1→2; genuine static-unreachable surface is still only
  the benign `trivia` pocket — evidence in `description`;
  `sv_preprocessor_syntax_closure_gate` now passes) and **A2** the
  stale `sv_preprocessor_quality_gate.sh` `pp_if_branch::root/s0`
  assertion re-targeted to the post-lift `pp_if_keyword::root` group
  (underlying coverage `[7,6]` genuinely satisfies — not weakened).
  `.2` split into `.2.1` (done) + `.2.2` (the deeper closed-loop
  `reachable-branch universe drift`, frontier). Finding **A3**
  (SV-main `sv_parser_aggregate_contract_gate` replay-shadow
  rejections of valid SV — escaped idents / `export *::*;`) is the
  same root class as Finding C → folded into `.3` (SV-main hardening),
  not preprocessor scope. Decision (PNT-eligible, no user escalation):
  the grammar edits are correctness fixes; the honest path is to
  faithfully re-lockstep the downstream proof surfaces as deep as the
  cascade goes (the binding lesson
  [[feedback_grammar_edit_proof_gate_lockstep]]) — never revert
  correct fixes, never weaken an invariant to mask a real defect.
- `2026-05-17` (`PGEN-SV-EXH-PROOF-0004`, **`.2.2`**): the
  `reachable-branch universe drifted` failure is a **mis-specified
  gate invariant**, not a real defect (root cause documented):
  `summary.reachable_branches` is a burn-down DEBT metric
  (`stimuli_generator.rs:1589` skips `deficit==0`), so the Cat-A
  `macro_default_atom` factoring legitimately makes stage0 leave 10
  uncovered that stage1 covers (`covered_branches` 37→47;
  `reachable_branches` 10→0) — desirable, wrongly flagged by a
  byte-equality assertion. The true static universe
  (`total_rules=73`/`total_branches=50`/`reachable_rules=72`) is
  stage-stable everywhere. Fix = a **correction not a relaxation**:
  replaced the `reachable_*` cross-stage equality with (a) `total_*`
  stage-equality (the genuine static-universe invariant the author
  intended — strengthened, holds) + (b) `reachable_*` non-increasing
  burn-down (the real no-regression guarantee — still catches debt
  GROWING across stages). Verified: gate completes (`MAKE_RC=0`),
  unreachable surface confined to `trivia` pocket. Cascade continues:
  `.2` split adds `.2.3` (A4 — `parseability_parser_rejections_total=3`,
  3 closed-loop directive stimuli the refactored grammar
  self-rejects; genuine campaign-caused round-trip regression).
- `2026-05-17` (`PGEN-SV-EXH-PROOF-0005`, **`.2.3` premise
  correction**): tested the "campaign-caused round-trip regression"
  premise against the exact campaign diffs and **falsified it**.
  `git show a5da52f4` = a structurally-equivalent lift
  (`(kw_ifdef|kw_ifndef) C` → `pp_if_keyword C`,
  `pp_if_keyword:=kw_ifdef|kw_ifndef`): identical generated/parsed
  language. `git show 7228231b` = ONLY a `->` annotation change on
  `macro_formals` (production unchanged). Both **generatively inert**
  — cannot introduce a generator⊋parser hole. The shrunk repro for
  all 3 self-rejections is a bare backtick `` ` `` which the grammar
  correctly rejects (`non_directive_text` excludes `` ` ``). So
  `.2.3` is re-characterized: the `parser_rejections` 0→3 move has a
  **different, not-yet-identified root cause** (non-grammar pipeline
  evolution this session, or a pre-existing seed-sensitive
  generator⊋parser asymmetry). No code changed; honest premise
  correction recorded before deep work proceeds on a wrong basis
  (same discipline as the `-0001` re-scope + the `.2.2` mis-spec
  finding — test the premise, correct transparently).
- `2026-05-17`: **Code-Change Doctrine** — every grammar / contract /
  gate-script change in `.2`–`.6` is leaf-owned (real grammar gaps in
  `.3` split into sub-leaves).

## Open Questions

- `.2` (RESOLVED in `.2.1`): contract/calibration re-baseline (the
  POST-SV-AUDIT/SVPP factoring is legitimate structure; genuine
  static-unreachable surface unchanged = benign `trivia` pocket), NOT
  a grammar revert/change. Same answer applied to A2.
- `.2.2` (RESOLVED): mis-specified gate invariant (burn-down metric
  treated as static universe), not a closed-loop defect — corrected
  (true universe pinned on `total_*`; debt non-increasing on
  `reachable_*`); not masked.
- `.2.3` (OPEN, re-characterized `-0005`): the campaign grammar edits
  are PROVEN generatively inert, so they did NOT cause
  `parser_rejections=3`. What actually moved it 0→3? Bisect the
  **non-grammar** pipeline (stimuli_generator / codegen / parser-gen
  commits this session) and/or test whether it is a pre-existing
  seed-sensitive generator⊋parser asymmetry (bare-backtick emission).
  Then honest fix (grammar-harden or fix the generator; never loosen
  the hard `==0` precondition to mask a real self-rejection).
- `.3`: which commit regressed the external-corpus parse surface to
  `0/14` + the SV-main closed-loop replay-shadow (A3)? Triage owned by
  `.3` (not the baseline).

## Blockers

- `SV-EXH-PROOF.3`–`.6` are blocked on `SV-EXH-PROOF.2` completing
  (`.2.3` remains: 3 preprocessor closed-loop self-rejected directive
  stimuli keep the preprocessor zero-gap proof verdict red; SV-main A3
  separately blocks `sv_parser_family_status_gate` and is owned by
  `.3`).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `SV-EXH-PROOF` (setup) | decomposition vs workflow rules; Code-Change-Doctrine precursor | `pass — tree created (initial trio-port hypothesis)` |
| `2026-05-17` | `SV-EXH-PROOF` (re-scope) | empirical audit of the SV proof stack vs SV's own contracts | `pass — trio-port hypothesis falsified; re-decomposed to external_corpus_backed_proof_surface` |
| `2026-05-17` | `SV-EXH-PROOF.1` | canonical-target measurement of `sv_external_corpus_triage_gate` (0/14, genuine rejections verified via parse logs + fresh probe), `sv_syntax_closure_gate` (pass, healthy), clean standalone `sv_preprocessor_syntax_closure_gate` (exit 2, `unreachable_branches=13>3`), `sv_formal_exhaustive_closure_gate` (fails — aborts at Finding A), code-read of the hard-coded literal at `sv_formal_exhaustive_closure_gate.sh:245`; git provenance of the preprocessor regression | `pass — deterministic baseline recorded (docs/SV_EXH_PROOF_BASELINE.md); 4 findings dispositioned; LIVE drift corrected same-commit; tree re-planned to 6 leaves; no code changed` |
| `2026-05-17` | `SV-EXH-PROOF.2.1` | A1: re-baselined contract → clean standalone `sv_preprocessor_syntax_closure_gate` PASS (`status:pass`, `unreachable_branches:13`, `unreachable_rules:1`, `reachable_rules:72`); genuine static-unreachable surface confirmed = only `trivia` (1 rule + 3 branches, `reason=unreachable_from_entry`) ⊆ allowed pocket. A2: confirmed `pp_if_branch::root/s0` absent post-lift and `pp_if_keyword::root` `success_counts=[7,6]` (both polarity branches genuinely exercised) before re-targeting the assertion; re-ran `sv_preprocessor_zero_plausible_gap_proof_gate` → got past A1/A2, surfaced the deeper `.2.2` reachable-branch-universe-drift (stage0=10/stage1=0; `reachable_rules=72` stable) | `pass for .2.1 (A1+A2 correct, evidence-grounded, verified at their gate level; not weakened). `.2` NOT complete — `.2.2` deeper closed-loop regression remains; honestly recorded, not masked` |
| `2026-05-17` | `SV-EXH-PROOF.2.2` | Root-caused the drift via `stimuli_generator.rs:1567-1733` (`deficit==0 continue` → `reachable_branches` is a burn-down debt count); confirmed per-stage `total_rules=73`/`total_branches=50`/`reachable_rules=72` stage-stable while `covered_branches` 37→47 (burn-down working); git-blamed the equality assertion (`a243bfeb`, generic, calibrated when pre-refactor branch coverage was flat). Replaced mis-spec equality with `total_*` stage-equality + `reachable_*` non-increasing. Re-ran `sv_preprocessor_zero_plausible_gap_proof_gate`: `MAKE_RC=0`, gate completes, drift error gone, `observed_unreachable_rules=["trivia"]` ⊆ allowed; next layer surfaced: `parseability_parser_rejections_total=3` | `pass for .2.2 (mis-spec corrected + true universe strengthened; not masked — verified). `.2` NOT complete — `.2.3` (3 closed-loop self-rejected directive stimuli) remains; honestly recorded` |
| `2026-05-17` | `SV-EXH-PROOF.2.3` (premise test) | Inspected `git show a5da52f4` + `git show 7228231b` (the only campaign preprocessor edits): a5da52f4 = structurally-equivalent inline-alt→named-rule lift (identical language); 7228231b = `->` annotation-only change (production unchanged). Probe-confirmed bare `` ` `` is correctly rejected (`Parser did not consume full input at position 0`). Read `sv_preprocessor_zero_plausible_gap_proof_gate.sh:234` (hard `==0`, no baseline) | `premise FALSIFIED — campaign grammar edits generatively inert; `.2.3` is NOT a campaign grammar regression. Re-characterized (root cause = non-grammar pipeline change or pre-existing seed-sensitive asymmetry; deep bisect next). No code; honest correction before proceeding` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-EXH-PROOF` (setup) | `PGEN-SV-EXH-PROOF-0000` | tree created + activated (initial trio-port hypothesis) |
| `SV-EXH-PROOF` (re-scope) | `PGEN-SV-EXH-PROOF-0001` | hypothesis falsified; re-decomposed to the real gap |
| `SV-EXH-PROOF.1` | `PGEN-SV-EXH-PROOF-0002` | measured baseline + scope lock + LIVE drift correction + 6-leaf re-plan; frontier → `.2` (preprocessor regression prerequisite) |
| `SV-EXH-PROOF.2.1` | `PGEN-SV-EXH-PROOF-0003` | A1 syntax-closure contract re-baseline (v1→2) + A2 `pp_if_keyword` quality-assertion re-target; both evidence-grounded + verified at gate level; `.2` split (`.2.1` done / `.2.2` deeper closed-loop drift = new frontier); A3 folded into `.3` |
| `SV-EXH-PROOF.2.2` | `PGEN-SV-EXH-PROOF-0004` | A3' reachable-branch-universe-drift = mis-specified gate invariant (burn-down metric treated as static universe); corrected (`total_*` stage-equality strengthened + `reachable_*` non-increasing), verified `MAKE_RC=0`/not masked; `.2` split adds `.2.3` (A4 — 3 closed-loop self-rejected directive stimuli, new frontier) |
| `SV-EXH-PROOF.2.3` (premise correction) | `PGEN-SV-EXH-PROOF-0005` | exact campaign diffs prove a5da52f4/7228231b generatively inert → `.2.3` is NOT a campaign grammar regression; re-characterized (root cause = non-grammar pipeline / pre-existing seed-sensitive asymmetry; bisect next). Docs-only honest correction; frontier stays `.2.3` |

## Changelog

- `2026-05-17`: Created + activated (initial trio-port hypothesis).
- `2026-05-17`: Re-scoped — trio-port falsified; sole gap =
  `external_corpus_backed_proof_surface`.
- `2026-05-17`: **`.1` measured baseline complete.** Ground-truth
  measurement surfaced a regressed preprocessor syntax-closure
  (prerequisite blocker, this session's lockstep defect), a healthy
  SV-main static-closure, a `0/14` (not `10/14`) external-corpus
  parse surface (proven-false LIVE drift, corrected same-commit), and
  a hard-coded `surface_present=true`. Tree re-planned to 6 leaves;
  frontier → `.2` (preprocessor syntax-closure regression
  remediation). Code-Change-Doctrine-compliant (`.1` changed no code).
- `2026-05-17`: **`.2.1` done.** The preprocessor regression is a
  cascade of un-lockstepped downstream proof-stack expectations from
  the legitimate POST-SV-AUDIT.2.1/INLINE-ALT-FIX.1 grammar edits.
  A1 (syntax-closure contract re-baseline `max_unreachable_branches`
  3→13, v1→2) + A2 (`pp_if_branch::root/s0` → `pp_if_keyword::root`
  quality-assertion re-target) fixed, evidence-grounded, verified at
  their gate level (not weakened). `.2` split into `.2.1` (done) +
  `.2.2` (the deeper closed-loop reachable-branch-universe drift,
  frontier). A3 (SV-main aggregate replay-shadow rejections) folded
  into `.3`. Code change is leaf-owned (contract json + quality-gate
  script). `.2` NOT complete (umbrella still red at `.2.2`).
- `2026-05-17`: **`.2.2` done.** The `reachable-branch universe
  drifted` failure was a **mis-specified gate invariant**, not a
  defect: `summary.reachable_branches` is a burn-down debt count
  (`stimuli_generator.rs:1589`), so the Cat-A factoring legitimately
  burned it down 10→0 (covered_branches 37→47) — wrongly flagged by a
  byte-equality assertion. Corrected (not relaxed): `total_*`
  stage-equality (true static universe — strengthened) + `reachable_*`
  non-increasing (genuine no-regression). Verified `MAKE_RC=0`, gate
  completes, unreachable surface confined to `trivia`. Cascade
  continues: `.2` split adds `.2.3` (A4 — preprocessor closed-loop
  `parser_rejections_total=3`: 3 self-rejected directive stimuli from
  the refactor, genuine round-trip regression, new frontier). Code
  change leaf-owned (`sv_preprocessor_aggregate_contract_gate.sh`).
  `.2` NOT complete (zero-gap proof verdict still red at `.2.3`).
- `2026-05-17`: **`.2.3` premise corrected (no code).** Tested the
  "campaign-caused round-trip regression" premise against the exact
  diffs and falsified it: `a5da52f4` (SVPP-0001) is a
  structurally-equivalent rule lift and `7228231b`
  (POST-SV-AUDIT.2.1) is a `->` annotation-only change — both
  generatively inert, so the campaign grammar edits did NOT cause
  `parser_rejections=3`. `.2.3` re-characterized: root cause is a
  separate not-yet-identified non-grammar pipeline change or a
  pre-existing seed-sensitive generator⊋parser asymmetry (bare-`` ` ``
  emission). Honest correction recorded before deep bisect; frontier
  stays `.2.3`.
