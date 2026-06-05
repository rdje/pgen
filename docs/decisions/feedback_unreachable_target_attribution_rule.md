---
name: feedback-unreachable-target-attribution-rule
description: When the stimuli generator fails to reach a target (branch/rule/EBNF fragment), the cause is EXACTLY ONE of two — generator deficiency OR an ill-formed EBNF — and must be attributed, never silently accepted. The linter is the adjudicator; order of suspicion is GRAMMAR-FIRST (unreachability is first a well-formedness signal, not generator weakness).
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-06
---

**Director directive (2026-06-06), the operational form of the linter⟷generator
DUALITY.** When the stimuli generator FAILS TO REACH a target — a branch, a rule,
or any EBNF fragment — there are EXACTLY TWO possible causes, and the failure MUST
be attributed to one. It is NEVER silently accepted as a "residual":

1. **Generator deficiency** (a constructor gap) — the target genuinely IS reachable,
   so the generator must be improved to witness it; OR
2. **EBNF not well-formed** — the target is genuinely UNREACHABLE (a dead branch/rule),
   so the fix belongs in the GRAMMAR, not the generator.

**The linter is the ADJUDICATOR**: for the decidable cases it PROVES which of the two
it is. The undecidable remainder is flagged loudly on that exact target for manual
adjudication — still never silently accepted.

**⚠️ Order of suspicion = GRAMMAR-FIRST.** An unreachable target is FIRST a
well-formedness signal about the grammar — NOT evidence that the generator is weak.
This INVERTS the historical SV literal-0 instinct (which chased the generator). So:
when the generator can't reach something, run the linter BEFORE adding generator
machinery — chasing a linter-proven-dead branch in the generator is wasted effort
that can never succeed; the dead branch must be fixed at the source. Only once the
linter confirms the target really IS reachable is it a generator gap. (Also the
cheaper, more-correct fix — composes with [[feedback_prefer_grammar_leave_engine_alone]].)

**Consequence:** literal-0 stimuli coverage stops being a number to drive down and
becomes a THEOREM — coverage is complete exactly when, for every target, the two
proofs agree (every reachable target witnessed, every unreachable target removed at
the source). Every uncovered target is a TICKET assigned to grammar OR generator,
never a shrug.

**First worked example (PGEN-GRAMMAR-WELLFORMED-0010, A2.1.1):** the boolean-abbrev
`?`-per-arm bug. The generator couldn't reach 7 SV branches (`consecutive_repetition`,
`boolean_abbrev*`); the `A2` always-succeeds-shadowing linter PROVED them unreachable
(each arm wrapped `( X )?` always succeeds → later arms dead); blame = EBNF
well-formedness (a likely LRM-PDF→.ebnf extraction artifact); fix = drop the spurious
`?` (the real optionality already lives at the caller `( boolean_abbrev )?`). always_matches
52→45 after the fix.

Owned by the **GRAMMAR-WELLFORMED** tree (`docs/tasks/GRAMMAR-WELLFORMED.md`). Book:
the "attribution rule" + "worked example" sections of `docs/book/src/grammar-wellformedness.md`.
Reinforces [[feedback_corpus_expected_from_spec_not_fix]] (never game the metric — a
removed dead branch is a CONSEQUENCE of a real grammar fix, not target-list trimming).
