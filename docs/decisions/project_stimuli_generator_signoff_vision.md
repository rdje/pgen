<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/project_stimuli_generator_signoff_vision.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: project-stimuli-generator-signoff-vision
description: "User vision (2026-05-31): make the EBNF-based stimuli generator signoff-grade — breathtaking, powerful, versatile, one of the best. Requires out-of-the-box thinking + a deliberate capability-gap audit, NOT just incremental coverage fixes."
metadata: 
  node_type: memory
  type: project
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

User, 2026-05-31: "At some point we will need to see what features the stimuli_generator still does NOT have, to make it really powerful, versatile and one of the best EBNF-based stimuli generators. We will need to think out of the box to make a signoff EBNF-based stimuli generator. It will need to be breathtaking." (user typo-corrected 2026-05-31: "breathtaking", one word)

**What this is:** a forward-looking strategic directive (NOT an immediate task). It elevates `rust/src/ast_pipeline/stimuli_generator.rs` from "a component that happens to work" to a first-class deliverable that must reach signoff/best-in-class quality.

**How to apply:**
- This is the natural umbrella that the current `SV-EXH-PROOF.7` work (close `focused_replay_target_debt_zero` — drive the target-driven generator to full grammar coverage) feeds INTO. `.7` exposes concrete generator limitations (e.g. 358 branches `never_selected` despite heavy coverage-guidance bias → likely structural: depth/parent-path reachability ceilings). Each such limitation is a data point for the capability-gap audit.
- When `.7` (or a natural pause) is reached, do a deliberate **capability-gap audit**: enumerate what a signoff EBNF-based stimuli generator SHOULD do that ours doesn't yet. Candidate axes to think through (out-of-the-box, not limited to): exhaustive/guaranteed branch+rule coverage (not best-effort); recursion-depth strategy that still reaches deep-but-rare branches; constraint-aware / semantic-store-aware generation (valid-by-construction stimuli); shrinking/minimization of counterexamples; coverage-closure proofs (the generator itself proving it covered everything reachable); determinism + seed reproducibility; performance at scale; multi-profile/dialect awareness; targeted/biased generation toward user-specified rules; negative/invalid stimuli generation for robustness; parity between in-memory and generated-module stimuli; cross-parser generality (must stay parser-AGNOSTIC per [[feedback_ast_pipeline_parser_agnostic]]).
- MUST stay parser-agnostic: every generator capability is a GENERAL grammar-structure property, never hardcoding rule names/sigils (per [[feedback_ast_pipeline_parser_agnostic]] — reaffirmed by user "utmost importance"). Every change task-tree-owned + signoff-verified ([[feedback_always_signoff_decisions]]).
- Likely warrants its own task tree (e.g. `STIMULI-SIGNOFF` or similar) once the scope is audited; until then, capture observed gaps as they surface during `.7`.

Related: [[feedback_correctness_before_speed]] (correctness first), [[feedback_universal_semantic_store]] (the store is available for context-aware generation), [[project_vision_and_discipline]].
