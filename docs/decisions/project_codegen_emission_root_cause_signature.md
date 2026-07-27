---
name: project-codegen-emission-root-cause-signature
description: DOCTRINE EXTENSION (2026-07-27, session #215) — the task-acceptance ROOT-CAUSE gate gains a FOURTH diagnosis-signature group for CODEGEN-EMISSION defects (`GENERATED-CLIPPY-CORRECTNESS:`, `clippy::<lint>`, `PGEN_CLIPPY_GENERATED_STRICT`), and every signature is now BOX-SCOPED — it must sit inside the ticked box's own bullet, closing the cross-file and incidental-prose leaks.
metadata:
  node_type: memory
  type: project
---

**What changed.** Two hardenings to `scripts/check_diagnosis_evidence.sh`, both owned by
`GENERATED-LINT-CORRECTNESS.3`.

**(1) A FOURTH signature group — CODEGEN-EMISSION.** `DIAGNOSIS_SIG` gained
`GENERATED-CLIPPY-CORRECTNESS: | clippy::[a-z_]{3,} | PGEN_CLIPPY_GENERATED_STRICT`. Group 1 is
correctness-defect diagnosis (`CERTIFICATE-COVERAGE:`, `[plannable-probe]`, `furthest_position=`,
`--trace-rules`, …); group 2 is performance (`self-time`, `flamegraph` —
[[project_speed_phase_profiler_root_cause_signature]]); group 3 is build integrity (`error[EXXXX]`,
`could not compile` — [[project_build_integrity_compiler_root_cause_signature]], the precedent this
extension follows exactly).

*Why.* A defect where the GENERATOR emits the wrong CODE is a fourth family with no instrument in
the existing list: there is no parse to trace (the parser is correct), no run to sample (it is not a
slowness defect), and no compiler error (the emission compiles fine — that is what made the 291
`clippy::eq_op` / `overly_complex_bool_expr` errors survive so long). The WHY+WHERE is the emission
site in the generator plus a census of the emitted artifacts, and the instrument that REPORTS it is
the generated-parser lint lane. `GENERATED-LINT-CORRECTNESS.2` measured the consequence: a fully
evidence-backed leaf FAILED the check because its genuine root-cause instrument matched no token.

*Why these tokens.* Same discipline as group 3 — verbatim tool output, not prose.
`GENERATED-CLIPPY-CORRECTNESS:` is the literal prefix every line of
`rust/scripts/generated_clippy_correctness_gate.sh` emits (the codegen analogue of
`CERTIFICATE-COVERAGE:`). `clippy::<lint>` requires a real lint path — the codegen analogue of
`error[EXXXX]`. `PGEN_CLIPPY_GENERATED_STRICT` is the generated stage's strict switch. Deliberately
EXCLUDED as too loose: bare `generated/…_parser.rs` (it appears in ordinary prose across many
leaves) and `make focus_<grammar>` (it appears in every regeneration recipe) — the easy but
gate-weakening choices.

**(2) BOX-SCOPED evidence.** The signature backing a ticked box must now sit inside THAT BOX'S OWN
bullet — its header line through to the next checklist box at the same-or-shallower indent, the next
markdown heading, or EOF. Previously the box test and the signature test were independent greps over
ALL staged `docs/tasks/*.md`.

*Two MEASURED leaks, both closed by this one mechanism.*
- **Cross-FILE.** A co-staged, unrelated tree file could supply the signature for a leaf carrying
  none. Measured: that is exactly how `GENERATED-LINT-CORRECTNESS.1` passed — its commit also staged
  `docs/tasks/QUANT-PLUS-ITER.md`, which carries such tokens for its own unrelated reasons. Routed to
  `.3` by `.2`.
- **Incidental PROSE.** A whole-file grep matched a token mentioned anywhere in the leaf rather than
  in the ticked box. This was already recorded as a known gap and deferred in
  [[project_build_integrity_compiler_root_cause_signature]] ("Watch item": *"the underlying looseness
  (whole-file grep, not box-scoped) is a known soundness gap in all three signature groups and is
  worth a future hardening slice — scope the grep to the ticked box's own bullet"*). This IS that
  slice; finding it first, before designing, is [[feedback_read_prior_art_before_designing]] working.

*A bug this leaf's own RED probe caught.* The first implementation matched the box KEYWORD against
the whole box body, so a box that merely MENTIONED "root cause" in its prose could stand in for the
real ROOT CAUSE box — probe RED-2 went green when it should have gone red. The keyword is now matched
against the box's HEADER LINE only (reusing the original, proven header regex); the signature is
matched against the body. **Evidence that the probes were worth writing: they failed the first
implementation.**

**Verification (RED/GREEN/CONTROL, all six arms).**
`docs/tasks/artifacts/generated_lint_correctness/run_diag_evidence_probes.sh` builds throwaway git
repos so staging is real: RED-1 (signature only in a co-staged leaf) BLOCKS; RED-2 (signature in the
file but outside the box) BLOCKS; RED-3 (box unticked) BLOCKS, unchanged; GREEN-1 (correctness-family
signature inside the box) ALLOWS; GREEN-2 (codegen-emission signature inside the box) ALLOWS — it
would have been BLOCKED before this change; CTRL-1 (no code staged) ALLOWS. 6/6.

**Scope / limits — stated, not hidden.**
- **The residual leak is WITHIN a file, across LEAVES.** A tree file holds many leaves' checklists,
  and the check cannot know which leaf a commit belongs to, so an OLDER leaf's qualifying box can
  still satisfy a NEWER leaf in the same file. Closing that needs per-leaf ownership the enforcer has
  no way to determine mechanically. Strictly better than before; not airtight.
- **A measured friction cost.** Replaying the new rule over every tracked task file: **56 carry a
  ticked ROOT CAUSE box, 26 are box-scoped-backed, 30 are not.** Those 30 are already-committed
  leaves and the rule binds only NEW commits — but the number is the honest price of the tightening
  and it is recorded rather than discovered later.
- ⭐ **A FIFTH family is visible in those 30 and is deliberately NOT added here.** Many cite a
  `file.rs:NNN` plus the culprit expression, or a controlled 3-arm differential — genuine WHY+WHERE
  that no signature group models. Widening the gate to accept "any file:line" would weaken it to
  "cite a line number", so the question is ROUTED (`GENERATED-LINT-CORRECTNESS.4`) rather than
  answered by quietly loosening the regex.
- `scripts/*` is outside the enforcer's own `code_changed` trigger set
  (`grammars/*.ebnf | rust/src/* | generated/* | ast_shape_contract/*.json`), so this edit did not
  itself require an acceptance checklist. Surfaced to the director as a doctrine evolution, exactly as
  the profiler and compiler extensions were. **Ratification pending** — unlike groups 2 and 3, which
  are settled doctrine, group 4 and the box-scoping are recorded here as applied-and-surfaced.
