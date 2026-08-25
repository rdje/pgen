---
name: a-control-defined-by-enumerating-a-domain-expires-when-the-domain-widens
description: "DISCIPLINE (2026-08-25, ENGINE-UNIVERSAL-SERVICES.46 slice 4) — a control that expresses its premise by ENUMERATING a domain (every declared profile, every known kind, every registered family) stops being a control the moment the domain gains a member, and NOBODY EDITS IT for that to happen. Measured: `the_same_shape_with_the_gate_removed_is_silent` spelled *gate removed* as `@profiles: [wide, narrow]` — a gate admitting every DECLARED profile — and `.46`(c) added an UNDECLARED-profile sentinel to the universe one commit later. A rule gated to every declared profile is still ABSENT under an undeclared spelling, so the control fired, correctly, on a premise that had silently become false. ⛔ The cheapest remedy — relax the assertion to the new count — would have RETIRED the only arm proving the detector keys on the gate rather than on the presence of a lookahead. Spell the premise INTENSIONALLY (no annotation at all) and pin the vacated enumeration case as its own arm."
id: a-control-defined-by-enumerating-a-domain-expires-when-the-domain-widens
title: "A control defined by enumerating a domain expires when the domain widens — and nobody edits it for that to happen"
date: 2026-08-25
evidence: docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .46 slice 4 sections [5] and "THE FIX FOR THE GREEN CONTROL IS THE INTENDED REMEDY"; rust/src/ast_pipeline/grammar_wellformedness.rs tests `the_same_shape_with_the_gate_removed_is_silent` (fixture now takes `Option<&str>`; `None` = genuinely ungated) and `a_gate_admitting_every_declared_profile_still_fires_under_the_undeclared_sentinel` (the vacated case, pinned on purpose); the founding measurement is `cargo test --features "generated_parsers ebnf_dual_run" --lib` reading 1123 passed / 4 failed on eef4f432
reverify: "cd rust && cargo test --features 'generated_parsers ebnf_dual_run' --lib grammar_wellformedness 2>&1 | grep -q 'a_gate_admitting_every_declared_profile_still_fires_under_the_undeclared_sentinel' && echo ENUMERATION-CASE-STILL-PINNED   # the arm exists only because the old control's premise expired; its absence means the enumeration case went unwatched again"
answers:
  - "why did a test I never touched start failing"
  - "how do I write a control that survives the domain growing"
  - "my green control fired and the finding looks correct — now what"
  - "is relaxing the assertion the right fix when a control goes red"
  - "what breaks when I add a sentinel or a new member to an enumerated set"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-25
---

## The rule

A control's premise must be stated **intensionally** — by the property — not **extensionally**, by
listing the domain's current members.

| premise | spelled extensionally | spelled intensionally |
|---|---|---|
| "this rule is not gated" | `@profiles: [every, declared, profile]` | ⭐ **no `@profiles` annotation at all** |
| "exactly one profile can exhibit this" | `assert_eq!(issues.len(), 1)` | assert *which* profiles, by name |

The extensional spelling is true on the day it is written and silently false afterwards. Nothing in
the control changes; **the domain underneath it moves.**

## What was measured

`ENGINE-UNIVERSAL-SERVICES.46` (c) added `UNDECLARED_PROFILE_SENTINEL` to the profile universe — a
real runtime state (`set_grammar_profile` passes an unknown spelling through un-coerced) in which
*every* gated rule is absent at once. One commit later, three unit tests were red:

| arm | how its premise was spelled | what the widened domain did |
|---|---|---|
| ⭐ `the_same_shape_with_the_gate_removed_is_silent` — **the GREEN CONTROL** | "gate removed" = `@profiles: [wide, narrow]` | a rule gated to every *declared* profile is still ABSENT under an *undeclared* one ⇒ the arm fired |
| `a_negative_lookahead_..._under_the_profile_that_gates_it` | `assert_eq!(issues.len(), 1)` | the sentinel adds a second, equally correct finding |
| `a_lookahead_nested_under_a_quantifier_is_still_found` | `assert_eq!(issues.len(), 1)` | same arithmetic |

All three findings were **correct**. All three assertions were **stale**. The difference matters:
this is not a regression to revert, it is a premise to re-derive.

## ⛔ The trap on the way out

The cheapest way to make a red control green is to update its expected count. Do that here and the
arm still passes, still has a reassuring name — and no longer tests anything, because
`the_same_shape_with_the_gate_removed_is_silent` exists to prove the detector keys on **the gate**
rather than on **the presence of a lookahead**, and a gate admitting every declared profile is a
gate. The count is not the premise. The premise is *gate-free*.

⇒ **When a control goes red, ask whether its PREMISE is still what its NAME says.** If it is not,
the fix is in the fixture, not in the assertion. Then pin the case the control just vacated —
"a gate admitting every declared profile" is now a real and interesting state, and it deserves an
arm that expects the finding rather than one that is surprised by it.

## Why no existing safeguard caught it

The three arms were written as one-difference pairs, with declared expectations, a RED direction and
a GREEN direction — everything [[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]]
and [[feedback_instrument_needs_ground_truth]] ask for. None of it helps, for a structural reason:
**every one of those disciplines checks the arm against the domain as it was when the arm was
written.** There is no safeguard inside a test for "the universe this test quantifies over grew."

The only thing that catches it is *running the suite*. Which is the second half of this record.

## ⛔⛔ The multiplier: these were red on HEAD for a whole commit, because NOTHING RAN THEM

```
grep -rn 'cargo test' .github/workflows/ rust/scripts/sota_exit_gate.sh scripts/check_doctrines.sh
# → nothing
```

No CI workflow, no doctrine enforcer and no gate — the flagship aggregate included — ran the suite,
and the three gates the offending commit ran are all `parse_harness_*`. The module carrying the arms
for the very feature it changed was run by none of them. A fourth test, unrelated, had been red since
2026-07-31 — **403 commits and 25 days**. Closed by `make -C rust lib_unit_test_gate`, wired as a
required check of `sota_exit_gate`; `GATE-REACHABILITY` refused the target as *"invoked by NOTHING"*
until it was, which is the same law one level up.

⛔⛔ **HONEST BOUND, and it is a correction this record earned the hard way.** The first write-up said
*"every `cargo test` in `rust/Makefile` is filtered to a named module"* — derived from three paths and
one file. A director challenge sent me to the closed census (`git ls-files -z | xargs -0 grep -l`
over the whole tree), which says otherwise twice: `test-parser` (`rust/Makefile:575`) runs a bare
unfiltered `cargo test`, and **14** tracked `docs/tasks/artifacts/*/battery.sh` batteries run the
exact full featured suite as a `libsuite` step. Neither is a gate, nothing invokes either, and
`test-parser` runs on DEFAULT features where 12 fail and 9 of those are feature-gating refusals — so
the *effect* is unchanged and the *sentence* was wrong. ⇒ **"nothing runs X" is a census claim about
a population you must first close**, and this record was written in the same commit that failed to
close it.

## Practical checks

- **Grep your fixtures for domain enumerations** whenever you add a member to a domain — a profile,
  a kind, a family, a sentinel. The blast radius of "I added one more case to an enum" includes
  every control whose premise is "all of them".
- **Prefer absence to exhaustive presence.** "No annotation" cannot be outgrown; "every annotation"
  can.
- **Assert identities, not cardinalities.** `assert!(profiles.contains(&"narrow"))` survives the
  domain growing; `assert_eq!(len, 1)` does not, and its failure message tells you nothing about
  which member is new.
- **When you widen a domain, add the arm that pins the new member's behaviour in the same commit** —
  otherwise the only record that the new state exists is the arms it broke.

Sibling of [[feedback_ask_the_instrument_the_question_its_founding_case_cannot_answer]] — that one is
about a set of positive declarations being unable to name its own complement, which is *why* the
sentinel had to be added; this one is about what adding it does to everything that had enumerated the
set. See also [[a-check-whose-inputs-all-pass-has-not-been-tested]].
