---
name: project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier
description: "MEASURED ENGINE LAW (2026-08-13, ENGINE-UNIVERSAL-SERVICES.17 slice 1): PGEN gives back at a CHOICE and refuses to at a QUANTIFIER. The default @branch_policy is longest_match — every alternative is evaluated and the longest wins, and a successful-but-losing alternative is retried when a later element fails — while the quantifier is possessive (Perl `a*+`) and never revisits its iteration count. ⇒ a starved trailing element is an asymmetry between PGEN's OWN combinators, NOT a property of PEG, and 'a multi-attempt protocol costs too much' is refuted by the tournament the engine already runs everywhere. The second non-negotiable is satisfied here by STATIC ELISION at codegen, not by refusing to ever attempt twice."
id: project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier
title: "PGEN gives back at the choice and not at the quantifier — the asymmetry is the engine's, not PEG's"
date: 2026-08-13
evidence: docs/tasks/artifacts/engine_universal_services/quantifier_policy/ (probe.sh, 7 self-checking controls); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .17 slice 1; rust/src/ast_pipeline/ast_based_generator.rs:4273/:4387/:4556/:6041-6091; rust/src/ast_pipeline/semantic_directive_registry.rs:59-71
reverify: "bash docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh | grep -q '7/7 as declared' && echo GIVE-BACK-LAWS-HOLD"
answers:
  - "does PGEN backtrack"
  - "is PGEN's ordered choice first-match commit or longest match"
  - "why does my trailing element never match after a star"
  - "is PGEN's quantifier greedy lazy or possessive"
  - "can I make a PGEN quantifier give an iteration back"
  - "how do I stop a greedy quantifier from eating the token the next element needs"
  - "does a multi-attempt protocol violate PGEN's peak-speed non-negotiable"
metadata:
  node_type: memory
  type: project
  created: 2026-08-13
---

**The question `ENGINE-UNIVERSAL-SERVICES.17` had to settle before designing anything:** PGEN's
generated parsers refuse `initial k = 8'(1);` once the cast chain is absorbed, because the greedy
`( suffix )*` consumes the `'(1)` that the enclosing `cast` still needs. The leaf's design note
attributed this to PEG — *"PEG defines `e*` as `A ← e A / ε`, and PEG's ordered choice commits once
an alternative succeeds"* — and concluded that any fix *"reintroduces exactly the backtracking PEG
removed to buy its memoization guarantee."*

That is a claim about the engine, so it was measured
([[feedback_read_prior_art_before_designing]]: *"when citing engine behaviour, re-measure it"*).

## THE LAWS, as measured

| grammar | input | verdict | law |
|---|---|---|---|
| `( "a" )* "a"` | `aaa` | **REJECT** | the quantifier is **possessive** — never gives an iteration back |
| `( "a" \| "ab" ) "c"` | `abc` | **ACCEPT** | the **choice gives back** a successful-but-losing alternative |
| `( "a" \| "ab" )` | `ab` | **ACCEPT** | the default `@branch_policy` is `longest_match`, not first-match commit |
| `( "a" &"a" )* "a"` | `aaa`, `a` | **ACCEPT** | a per-iteration **stop-guard** closes the starvation with no re-entry |
| `star_rule := ( "a" &"a" )*`, no residual | `aaa` | **REJECT** | …but that guard is context-dependent, so it must be **call-site** scoped |

The first two are a deliberate **one-difference pair**: both are *"a sub-match succeeds, then the
element after it starves"*, and the only variable is which combinator produced the sub-match.

## THE THREE CONSEQUENCES

1. ⛔ **"PGEN is a PEG, so it cannot give back" is false.** It is a PEG in its *determinism* property
   — one winner per choice, never a parse forest — and explicitly **not** in its selection rule. The
   emitted code says so: *"Multi-branch - evaluate all branches and keep the longest successful
   match"* (`ast_based_generator.rs:4273`), selection ordered priority → longest → associativity
   (`:4387`). Classical first-success commit exists but must be asked for
   (`@branch_policy: ordered`). ⇒ a starved trailing element is an asymmetry between PGEN's own two
   combinators, and the fix is scoped to one of them.
2. ⛔ **"A multi-attempt protocol violates the peak-speed non-negotiable" does not survive
   measurement either.** The engine already attempts every alternative at every non-degenerate
   multi-branch rule. What keeps that affordable is **static elision** — codegen proves the
   tournament unnecessary and emits a degenerate dispatch instead (`:4556`) — and the same pattern
   appears at the quantifier as the `RGX-0078.5.i.7` Q-GUARD (`:6006`), a codegen-computed byte-set
   test that breaks the loop with furthest-position parity. ⇒ *costs are REJECTED, not traded* has
   been implemented here as **"prove the cost away at generation time"**, never as *"never attempt
   twice"*. A design is not required to show zero attempts; it is required to show the extra
   attempts are statically confined to the sites that need them.
3. ⭐ **The author-facing law, and its bound.** A greedy quantifier followed by an element that can
   also start an iteration will starve it; the PEG-native repair is
   `( !close body )* close` — or, when the holder needs exactly one more, `( body &FIRST(close) )*`.
   ⛔ It is **context-dependent**: written onto the rule it breaks every caller that wants the whole
   run (row 5 above), so it belongs on the **call site**. For an engine-synthesized repair that means
   a sheared clone reached only from the holder.

## WHAT THIS DOES *NOT* SAY

- It does **not** license a re-enterable `*`. A stop-guard leaves every rule with exactly one result
  per position, so the memo is untouched; a re-enterable `*` does not, and that memo exposure is the
  load-bearing risk the leaf's note correctly identified. The two options carry different burdens and
  only one of them has been priced.
- It does **not** reopen runtime left-recursion support
  ([[project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime]]) — the
  blocker is a quantifier's give-back policy, not the elimination mechanism.
- The controls are two-terminal reductions. They pin the **laws**; they say nothing about whether a
  guard is statically computable on the shipped SystemVerilog grammar, which needs
  `--report-indirect-lr-plan` on the real thing.

## PRIOR OCCURRENCES (the class is recurrent, and was named before)

- `SV-EXH-PROOF.3.3.4.b.2` — `hierarchical_identifier` starved by the same law; a
  `!callable_method_call_body` stop-guard was applied and **reverted on a director decision** in
  favour of a general engine service over a per-rule patch.
- [[feedback_layer_0_unified_quantifier]] — Layer 0's own closing section names *"cross-rule
  backtracking"* as the class it deliberately does **not** fix.
- `GRAMMAR-WELLFORMED` — `macro_default_value := macro_default_atom+`, a third victim in a third
  family.
- `LANG-CAPABILITY-AUDIT.3b` FINDING 3 — measured the possessive law and recorded the
  `( !close body )* close` repair as working.

Related: [[project_north_star]], [[feedback_read_prior_art_before_designing]],
[[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]],
[[project_capability_growth_is_zero_cost_and_neutral]].
