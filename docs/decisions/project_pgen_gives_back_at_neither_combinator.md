---
name: project_pgen_gives_back_at_neither_combinator
description: "CORRECTED ENGINE LAW (2026-08-13, ENGINE-UNIVERSAL-SERVICES.17 slice 4) — supersedes project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier. PGEN gives back at NEITHER combinator. The choice keeps ONE winner (longest by default) in a single `best_content` slot and discards every losing alternative, so there is nothing to retry when a later element fails; the quantifier keeps the maximum count. The earlier record's one-difference pair was NON-DISCRIMINATING: `( \"a\" | \"ab\" ) \"c\"` on `abc` accepts because the longest alternative is also the one that works. The discriminating shape `ch := \"a\" | \"ab\"` with `scratch := ch \"bc\"` REJECTS `abc`. ⇒ a starved follower is NOT an asymmetry between PGEN's combinators — both commit — and the repair is a CALL-SITE follow restriction in TWO positions (per-iteration and trailing) which close DISJOINT starvations."
id: project_pgen_gives_back_at_neither_combinator
title: "PGEN commits at every combinator and gives back at none — and the repair is a two-position call-site follow guard"
date: 2026-08-13
supersedes: project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier
evidence: docs/tasks/artifacts/engine_universal_services/guard_effectiveness/ (probe.sh, 37 self-checking rows on BOTH oracles); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .17 slice 4; rust/src/ast_pipeline/ast_based_generator.rs:5037 (the single `best_content` slot), :4380-4440 (the dethrone cascade), :5102 (the winner consumed), :6041-6091 (the possessive loop)
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh --interp-only | grep -q 'as declared' && echo COMMIT-LAWS-HOLD"
answers:
  - "does PGEN backtrack"
  - "does PGEN's choice retry another alternative when a later element fails"
  - "is PGEN's ordered choice first-match commit or longest match"
  - "why does my trailing element never match after a star"
  - "why does my trailing element never match after a rule with alternatives"
  - "is PGEN's quantifier greedy lazy or possessive"
  - "how do I stop a greedy quantifier from eating the token the next element needs"
  - "how do I stop an over-long alternative from eating the token the next element needs"
  - "where does a follow restriction go on an LR-eliminated rule"
metadata:
  node_type: memory
  type: project
  created: 2026-08-13
---

⛔⛔ **This record CORRECTS
[[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]]**, which is retained (superseded,
not deleted) so the audit trail shows which sentence moved and why.

## The law, as it actually is

| grammar | input | verdict | law |
|---|---|---|---|
| `ch := "a" \| "ab"` · `scratch := ch "bc"` | `abc` | **REJECT** | the **choice commits** — a successful-but-losing alternative is discarded, never retried |
| the same under `@branch_policy: ordered` | `abc` | **ACCEPT** | the parse EXISTS; the engine refused to reach it ⇒ the REJECT is the commit, not the grammar |
| `( "a" )* "a"` | `aaa` | **REJECT** | the quantifier is **possessive** — never gives an iteration back |
| `ch := "a" \| "ab"` · `scratch := ch "c"` | `abc` | **ACCEPT** | the superseded record's evidence, reproduced — and **non-discriminating** |

**Located.** The multi-branch tournament declares ONE winner slot —
`let mut best_content: Option<ParseContent<'input>> = None;`
(`ast_based_generator.rs:5037`) — and the policy cascade at `:4380-4440` only decides whether a
candidate *dethrones* the incumbent. The winner is consumed once at `:5102`. No losing alternative
is retained anywhere, so there is no structure a caller-failure could retry against. The possessive
loop is at `:6041-6091`, unchanged from the superseded record.

## Why the earlier record was wrong, precisely

Its one-difference pair was `( "a" | "ab" ) "c"` on `abc` ⇒ ACCEPT, read as *"`"a"` wins, `"c"`
fails, the choice gives back and retries `"ab"`."* But the record's own third row establishes that
the default policy is `longest_match`, so `"ab"` wins **outright** and `"c"` then matches the single
remaining byte. The parse completes on the first and only alternative the choice ever kept.

⇒ **the case is predicted identically by both hypotheses.** It is not weak evidence for the
give-back; it is no evidence at all. Making the following element two bytes instead of one (`"bc"`,
so only the SHORTER alternative can finish) separates them, and the answer flips.

⭐ **The transferable lesson.** The bank that carried this case was self-checking in both directions
and could not notice, because the expectation it checked (`ACCEPT`) was the right answer for the
wrong reason. A control that passes under both hypotheses does not discriminate — and a
*mechanism* may never be read out of a case that does not. Sibling of
[[a-check-whose-inputs-all-pass-has-not-been-tested]]: that one is about inputs that never exercise
a branch, this one about an output that never separates two explanations.

## What survives from the superseded record, and what does not

**Survives, unchanged:**

- The quantifier is possessive (Perl `a*+`), and that is the mechanism starving
  `initial k = 8'(1);`.
- *"Costs are REJECTED, not traded"* has been implemented here as **prove the cost away at
  generation time**, never as *"never attempt twice"* — the tournament does evaluate every
  alternative, and `degenerate_dispatch_byte_sets` (`:4556`) elides it where codegen can prove it
  unnecessary. Attempting is not the expensive thing; attempting where it was avoidable is.
- The repair is a **PEG-native follow restriction**, it is **context-dependent**, and it therefore
  belongs on the **call site** — for an engine-synthesized repair, a sheared clone reached only from
  the holder.

**Does not survive:**

- ⛔ *"A starved trailing element is an asymmetry between PGEN's own two combinators, so the fix is
  scoped to one of them."* Both combinators commit. There is no asymmetry, and the starvation has
  **two** independent sources that must both be repaired.
- ⛔ *"PGEN is a PEG in its determinism property and explicitly not in its selection rule, therefore
  it already gives back."* The first half is right and the conclusion does not follow: a
  longest-match tournament is a different *selection rule*, not a *retry protocol*.
- ⛔ Any argument for a re-enterable `*` that leans on *"restoring at the quantifier what the choice
  already does."* It would be the **first** give-back in the engine, and must be priced as new
  behaviour rather than as parity.

## The repair has TWO positions, and they close DISJOINT starvations

Measured on the post-rewrite shape with SystemVerilog's own nullable-`trivia` layout model
(`guard_effectiveness/`, 37 rows, both oracles):

```text
per-iteration  X := X_lr_base ( X_lr_suffix &( residual ) )*
trailing       X := X_lr_base ( X_lr_suffix )*             &( residual )
both           X := X_lr_base ( X_lr_suffix &( residual ) )* &( residual )
```

- **Per-iteration only** closes the LOOP starvation (`k = n'(n);`) and cannot touch the SEED
  starvation (`k = t'(n);`), where the loop takes zero iterations and the over-long match came from
  the rewritten base's own sheared clone.
- **Trailing only** closes the SEED starvation and cannot touch the LOOP starvation — by the time it
  runs, the possessive `*` has committed to the maximum count and there is no give-back to a shorter
  one.
- **Both** close all six shapes. ⇒ one clone chain carrying two lookaheads, not two clone chains.

⭐ **Why the trailing guard needs no give-back at the choice**, which is what makes it affordable:
refusing an over-long seed makes that branch **fail** rather than win, and a failed branch is not a
losing branch — the tournament simply has one fewer candidate and picks the next. The repair
converts a would-be give-back into a branch failure the existing selection already handles.

⛔ **And the guard must be BYTE-EXACT, not a FIRST-set byte test.** `trivia` is nullable and leads
every token, so `/` is in `FIRST` of every token; a byte-set guard passes on a comment exactly where
it had to refuse (`k = n'(n)/*c*/;`). `.17` slice 2 measured exactness at **0 of 129** sites on both
SystemVerilog grammars. The structural form is mandatory, at the price of one residual sub-parse per
committed iteration instead of one byte compare.

## Prior occurrences (unchanged from the superseded record)

- `SV-EXH-PROOF.3.3.4.b.2` — `hierarchical_identifier` starved by the same law; a stop-guard was
  applied and **reverted on a director decision** in favour of a general engine service over a
  per-rule patch.
- [[feedback_layer_0_unified_quantifier]] — Layer 0's closing section names *"cross-rule
  backtracking"* as the class it deliberately does **not** fix.
- `GRAMMAR-WELLFORMED` — `macro_default_value := macro_default_atom+`, a third victim in a third
  family.
- `LANG-CAPABILITY-AUDIT.3b` FINDING 3 — measured the possessive law and recorded
  `( !close body )* close` as the working repair.

Related: [[project_north_star]], [[feedback_read_prior_art_before_designing]],
[[left-recursion-is-an-engine-service-not-a-grammar-authoring-burden]],
[[project_indirect_left_recursion_is_eliminated_at_generation_not_grown_at_runtime]],
[[feedback_instrument_needs_ground_truth]].
