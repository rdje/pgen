---
name: project-duplicate-alternative-selection-semantics-conditional
description: The grammar-linter `DuplicateAlternative` shadowing verdict is sound ONLY where the rule's tournament tie-break provably keeps the EARLIER twin — under `@associativity: right`, a later-higher `@priority`, or `@deterministic_group` evaluation-order rotation the engine SELECTS the later "duplicate"; under `@associativity: nonassoc` an equal-priority tie FAILS the whole choice (so the verdict holds but the "merge or remove" remedy would change acceptance). Fixed in GRAMMAR-WELLFORMED.A2.4 by conditioning both the verdict and its `DuplicateOf` certificate on the effective selection semantics, resolved by the same shared functions codegen uses; A2.4 also tightened the A2.3 fixed-prefix condition to require no partition rotation. Completes the A2.2/A2.3/A2.4 arc: every ordered-choice deadness verdict now names AND checks the selection semantics it assumes.
metadata:
  node_type: memory
  type: project
  created: 2026-07-07
---

**Finding (2026-07-07, session #52 — `Protocol D` probes, `GRAMMAR-WELLFORMED.A2.4` fix).** The
last unconditioned ordered-choice deadness verdict, `ShadowingReason::DuplicateAlternative` ("two
identical alternatives — the later is unreachable"), carried the same latent assumption A2.2/A2.3
retired ([[project_earlier_always_matches_unsound_backtracking]],
[[project_fixed_terminal_prefix_policy_conditional]]): it assumed the earlier twin always wins. But
PGEN's `|` is a **branch tournament**, and for two equal-length twins the winner is decided by the
tie-break — `@priority` (compared first), then `@associativity` (`left` keeps the earlier, `right`
picks the later, `nonassoc` fails the choice on a tie), all under a `@deterministic_group`
evaluation-order **rotation** that can move either twin to the front.

**Decisive proof (Protocol D on the scratch slot — `scratch := "a" | "a"`, input `a`).** Four live
probes, each read from the engine's own `🏁 selected branch N/M` trace or accept/reject verdict:
- **(R)** `@associativity: right` → `🏁 selected branch 2/2 (… associativity=right)` — the engine
  SELECTS the later twin, while `--lint-grammar` hard-failed rc=1 "alternative #1 is unreachable";
- **(P)** `@priority: [0, 5]` (default assoc/policy) → `🏁 selected branch 2/2 (priority=5)` — the
  later, higher-priority twin wins;
- **(D)** `@deterministic_group: "spin"` (FNV offset 1, all-default assoc/policy) → `🏁 selected
  branch 2/2` — the rotation makes the later twin the incumbent that a `left` tie keeps;
- **(N)** `@associativity: nonassoc` → the twins REJECT `a` (`Backtrack at position 0`) while the
  deduplicated control `probe_n_single := "a"` ACCEPTS it — the nonassoc tie fails the whole choice,
  so *neither* twin is selected and the pair is load-bearing (removing one would flip to accept).

**Fix (A2.4, linter tier — zero parse-behavior change, byte-identical regen).** The duplicate
verdict and its `DuplicateOf` proof certificate now read the rule's effective `@associativity` /
`@priority` / `@deterministic_group` (and branch-phase-predicate surface) through a single
`RuleSelectionSemantics::duplicate_verdict` resolver, and fire only where the earlier twin provably
wins — `ordered` (first-success commit); or a tournament where the earlier twin's effective
priority is strictly higher, or ties break its way (`left`), with no branch-phase predicate and no
partition rotation. The `nonassoc` equal-priority tie gets its **own** reason
(`DuplicateAlternativeNonassocTie`): the later twin is still unreachable, but the message says
"restructure deliberately," not "merge or remove," because removing one duplicate would change
acceptance. The selection semantics are resolved by the shared
`effective_rule_associativity` / `effective_rule_branch_priorities` /
`effective_rule_deterministic_partition_policy` functions in `semantic_directive_registry.rs` — the
SAME ones codegen's tournament now delegates to (single source of truth; the two can never drift,
proven by byte-identical json + rtl_frontend + systemverilog regen). A2.4 also tightened the A2.3
fixed-prefix condition: partition rotation reorders `ordered` first-success, so the fixed-prefix
verdict now additionally requires no `@deterministic_group`.

**Why:** the certifying linter's contract is "never declares a live fragment dead"
([[feedback_certifying_linter_trustworthiness]]). Zero shipped grammars use `@associativity` and
the 13-grammar shadowing sweep stays at 0 findings, so like A2.3 this was a hard gate waiting to
false-block the first author who writes duplicate-tied alternatives under a non-default tie-break.

**How to apply (the general rule, now complete for ordered-choice deadness):** every unreachability
argument over an ordered choice assumes a selection semantics — name it, check it against the
engine's ACTUAL tournament (`generate_or_logic` tie-break: priority → associativity, under the
`evaluation_order` rotation), and make the certificate re-derive the condition, because a
certificate that re-verifies on structure alone can be re-verifiable yet false. With A2.2
(always-succeeds retired), A2.3 (fixed-prefix policy-conditioned), and A2.4 (duplicate
selection-semantics-conditioned), every surviving ordered-choice deadness verdict is now conditioned
on the semantics it assumes.
