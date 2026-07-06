---
name: project-fixed-terminal-prefix-policy-conditional
description: The grammar-linter `FixedTerminalPrefix` shadowing verdict is sound ONLY under `@branch_policy: ordered` (first-success commit) with no branch-phase predicates — under PGEN's DEFAULT `longest_match` (and `priority_first`) the engine's tournament SELECTS the later, longer alternative, so the unconditional verdict declared a live fragment dead. Fixed in GRAMMAR-WELLFORMED.A2.3 by conditioning both the verdict and its proof certificate on the rule's effective branch policy, derived by the same shared function codegen uses. General rule: every deadness verdict over an ordered choice must name — and check — the selection semantics it assumes.
metadata:
  node_type: memory
  type: project
  created: 2026-07-07
---

**Finding (2026-07-07, session #51 — `PARSE-HARNESS.8` evidence, `GRAMMAR-WELLFORMED.A2.3` fix).**
The linter's OTHER "PEG commits" shadowing verdict, `ShadowingReason::FixedTerminalPrefix`
("`a | a b` — the earlier alternative is a fixed-terminal prefix of the later, PEG commits, so the
later is unreachable"), carried the SAME false premise the retired `EarlierAlwaysMatches` check did
([[project_earlier_always_matches_unsound_backtracking]]): it assumed first-success commit, but
PGEN's `|` is a **branch tournament** whose selection semantics is the rule's `@branch_policy` —
`longest_match` (the DEFAULT; longest match wins, ties to the earlier), `ordered` (first-success
commit), or `priority_first` (`@priority` rank, longest-match tie-break).

**Decisive proof (the parse harness's first real use — PARSE-HARNESS.8).** On the scratch slot with
`scratch := "a" | "a" "b"` (no annotations = default policy), input `"ab"`:
- the engine ACCEPTS with the LATER alternative's typed AST, and its own trace reads
  `🏁 Rule 'scratch' selected branch 2/2 consuming 2 chars (… branch_policy=longest_match)`;
- `--lint-grammar` on the SAME grammar hard-failed **rc=1** claiming `alternative #1 is unreachable
  … (PEG commits to the earlier alternative)` — a false deadness verdict on a demonstrably-selected
  branch.
The `.6.1` combinator gate pins the full policy matrix on both implementations: only `ordered`
rejects `"ab"` (commits to the first alt); `longest_match` (default + explicit) and
`priority_first` accept it via the later alt. Zero shipped grammars use `ordered`.

**Fix (A2.3, linter tier — zero parse-behavior change, zero regen).** The verdict and its
`FixedTerminalPrefixBy` proof certificate are now **policy-conditional**: they fire/verify only when
the owning rule's effective `@branch_policy` is `ordered` AND the rule carries no branch-phase
`@predicate` (a branch predicate can block the earlier alternative after it matches, reviving the
later one). The policy is derived by `effective_rule_branch_policy` in
`semantic_directive_registry.rs` — the SAME function codegen's tournament uses (single source of
truth; the two can never drift). Under `longest_match`/`priority_first` the pattern is the normal
longest-match idiom: live, correct, and deliberately not even a note. The certificate checker
re-derives the policy condition, so a stale/policy-false certificate is rejected, not re-verified
on structure alone.

**Why:** the certifying linter's contract is "never declares a live fragment dead"
([[feedback_certifying_linter_trustworthiness]]). A verdict inert on every shipped grammar is still
a hard gate waiting to false-block the first author who writes the legitimate `a | ab` idiom.

**How to apply (the general rule):** every unreachability argument over an ordered choice
implicitly assumes a selection semantics. Name it, then check it against the engine's ACTUAL policy
table (`generate_or_logic` / `rule_branch_policy`) before gating on it — and make the certificate
carry the condition, because a certificate that re-verifies on structure alone can be re-verifiable
yet false. Open audit of the same class: `GRAMMAR-WELLFORMED.A2.4` (the `DuplicateAlternative`
tie-break inversion under `@associativity: right` / `nonassoc` — logged, unprobed).
