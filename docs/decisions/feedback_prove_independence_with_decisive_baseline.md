<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_prove_independence_with_decisive_baseline.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-prove-independence-with-decisive-baseline
description: Before recording "this failure is pre-existing / independent of my change", prove it with a decisive baseline experiment (stash the change, rebuild, re-measure) — never conclude independence from "I only touched X, and gate Y looks static".
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Lesson (SV-EXH-PROOF.2.3.2, 2026-05-18):** after the closed-loop
generator fix drove `parser_rejections → 0`, the proof gate then
failed its nested `sv_preprocessor_syntax_closure_gate`
(`unreachable_branches=24 > 13`). I reasoned: "I only modified
`stimuli_generator.rs`; the grammar is git-unchanged; syntax-closure is
a *static* analysis ⇒ this is pre-existing and independent of my
change" — and was about to record that. It was **wrong**: the
syntax-closure gate measures `reachable_branches` via a single
deterministic *stimuli-probe* sample, so a generator change DOES move
it. A decisive `git stash` baseline (stash the change, rebuild, re-run
the standalone gate) showed PASS (37/13) without the change vs FAIL
(26/24) with it — proving my change caused it. Only then could the
real, correct resolution proceed (an honest in-slice contract
re-baseline after proving zero new dead grammar).

**Why:** "I only touched X" + "gate Y is "static"/unrelated" is an
assumption about a dependency edge you have not verified. Gates often
have non-obvious inputs (a probe sample, a regenerated artifact, a
cached sidecar). Recording "pre-existing/independent" on that
assumption mis-scopes the work, can dump a real owned regression on a
future session, and violates the verify-don't-assume discipline.

**How to apply:**
- Before writing "pre-existing", "unrelated", or "independent of my
  change" about any failure, run the **decisive baseline**: revert
  *only* your change (e.g. `git stash push -- <file>`), rebuild the
  exact artifact, re-measure. Independence is established by the
  baseline reproducing the failure WITHOUT your change — not by
  reasoning about which files you edited.
- Treat "this gate is static / cannot depend on my change" as a
  hypothesis to falsify (inspect the gate's actual inputs), not a
  given. Here the syntax-closure gate's `reachable_branches` came from
  a `stimuli_count=1` probe.
- If the baseline proves you caused it, you own it in-slice
  ([[feedback_grammar_edit_proof_gate_lockstep]]); resolve honestly
  (re-baseline only after proving the genuine invariant is intact —
  e.g. the gap-report `reason=unreachable_from_entry` surface
  unchanged), never by masking.

Related: [[feedback_corpus_expected_from_spec_not_fix]] (the gate is
the arbiter, not your reasoning), [[feedback_ebnf_consult_annotation_docs]]
(verify the mechanism on the real artifact),
[[feedback_grammar_edit_proof_gate_lockstep]] (a code/generator change
owns ALL its downstream proof contracts same-slice).
