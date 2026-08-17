---
id: a-token-the-standard-writes-contiguously-must-be-one-terminal
title: A token the STANDARD writes contiguously must be ONE terminal — composing it out of grammar elements silently accepts the spaced spelling, because layout is skipped between elements
answers:
  - "the LRM writes this token with no spaces — do I model it as a sequence or as one regex"
  - "how do I stop my grammar accepting a whitespace-separated version of a fused token"
  - "my repair for a mis-transcribed terminal would over-accept — what is the shape that does not"
  - "when is a follow-restriction / lexical annotation actually needed and when is it not"
  - "the identifier grammar admits $ or . inside a name — what does that mean for a token containing one"
  - "how do I decide between a scannerless composition and a single terminal regex"
tags: [grammar, lexical, terminals, over-acceptance, systemverilog, lrm-fidelity, whitespace]
date: 2026-08-17
status: current
evidence: SV-CORPUS-GRAD.13c.2f slice 4. IEEE 1800 A.7.5 writes `PATHPULSE$specify_input_terminal_descriptor$specify_output_terminal_descriptor`; the LRM→EBNF extraction transliterated the `$` into the word `_dollar` and flattened both nonterminal references into the token text, so `pulse_control_specparam` was unreachable through BOTH alternatives and IEEE 1800-2023 §30.7.1's own example REJECTED at `furthest_position=166`. The planned repair was the composed form (`PATHPULSE$` descriptor `$` descriptor) plus a `[> … ]` follow restriction to stop it spanning whitespace; the composed form was never built, because `trivia` skips layout before every element and A.9.3 makes `PATHPULSE$clk$q` ONE simple identifier — so `PATHPULSE$ clk $ q` is three tokens and is not legal. One contiguous regex refuses it BY CONSTRUCTION. Measured after the fix - `PATHPULSE$ clk $ q = (2, 9);` REJECT, `PATHPULSE$clk$q = (2, 9);` ACCEPT, corpus 16 336 files with 3 rows improving and 0 regressing, accepts-invalid unchanged at 21.
reverify: "python3 stimuli/sv/run_adjudication_repros.py 2>&1 | tail -1   # invalid_pathpulse_spaced_descriptors.sv must stay REJECT; failures=0"
---

**Whitespace is skipped BETWEEN grammar elements and never INSIDE a terminal.** So the choice
between *one regex* and *a sequence of elements* is not a style question — it decides whether the
spaced spelling of the token is in your language.

## What happened

A grammar terminal had been mis-transcribed: the standard's literal `$` had become the word
`_dollar`, so the production could only ever fire on source containing the characters
`PATHPULSE_dollar`, which no real source does. The construct was dead, and the standard's own
example was rejected.

The obvious repair reads straight off the standard's own production —

```ebnf
# the composed form — DO NOT DO THIS
pulse_control_specparam := pathpulse_dollar specify_input_terminal_descriptor
                           dollar specify_output_terminal_descriptor assign lparen … rparen
```

— and it is wrong in a way a verdict test does not show, because it *accepts the intended input*.
It also accepts `PATHPULSE$ clk $ q = (2, 9)`, since layout is skipped before each of those four
elements. The planned mitigation was a declarative follow restriction (`[> … ]`) forbidding the
gap. That is a real feature and it was the wrong tool: the question is not *"may a separator appear
here"*, it is *"is this one lexical token"*.

## Why the single regex is the correct shape, not merely the cheaper one

Read the standard's **identifier** rule before deciding. IEEE 1800 A.9.3 gives
`simple_identifier ::= [a-zA-Z_]{[a-zA-Z0-9_$]}` — `$` is an identifier character. Therefore
`PATHPULSE$clk$q` lexes as **one** identifier, and the spaced spelling is a *different token
sequence*, not the same token with layout in it. A contiguous regex is then a faithful model:

```ebnf
kw_pathpulse_dollar := trivia /PATHPULSE\$/
kw_pathpulse_path   := trivia /PATHPULSE\$[a-zA-Z_][a-zA-Z0-9_$]*\$[a-zA-Z_][a-zA-Z0-9_$]*/
```

and the over-acceptance is refused *by construction* — there is no state in which the engine could
admit it, so nothing has to be enforced, gated or remembered later.

⭐ The clause text is what closes it. §30.7.1 adds *"the terminals may not be a bit-select or
part-select of a vector"*, which removes the only part of the descriptor that is not
identifier-shaped. **A production that looks un-regex-able in Annex A can be regex-able once the
clause's restriction is read** — the Annex is the shape, the clause is the constraint, and the
constraint is what decides the lexical class.

## The general rule

1. Ask what the **lexer** would do, not what the production diagram looks like. If the standard's
   own identifier/token rules make the whole span one token, model it as one terminal.
2. If it genuinely is several tokens that merely *must not* be separated, that is what a follow
   restriction is for. Those are different situations and only the first one is free.
3. Pin the spaced spelling as an **invalid** repro in the same commit
   (`stimuli/sv/adjudication_repros/invalid_pathpulse_spaced_descriptors.sv`). A fix that is correct
   by construction today can be re-composed by someone later; the guard is what makes it stay
   correct.

## The trap next door

⛔ **A verdict cannot tell you which alternative parsed.** The same input accepted before the fix
*and* after it, because it fell through to an ordinary assignment. The discriminator is the declared
AST `kind`, not the rule name — grepping the AST for `pulse_control` returns **0** in both states,
since no annotation emits the rule's own name. Pin the ARM, and prove the arm claim can go RED
against the route it must exclude. See
[[an-accept-is-not-evidence-the-alternative-under-test-fired]] and the tracked RED control
`docs/tasks/artifacts/sv_corpus_grad/es13c2f4_pathpulse/arm_red_control.py` (8/8, 4 of them RED).
