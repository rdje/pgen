---
id: a-profile-gate-inverts-every-negative-lookahead-on-the-gated-rule
title: A dialect gate silently inverts every negative lookahead on the gated rule — the strict profile gets MORE permissive
answers:
  - "I am about to @profiles-gate a rule — what breaks that no lint will tell me"
  - "why did my strict profile start accepting something the permissive one rejects"
  - "does a `!X` lookahead still refuse when X is gated out of the active profile"
  - "how do I gate a terminal out of a dialect without widening the language"
  - "my probe printed verdicts under two profiles and they were identical — is that a result"
  - "how do I tell a vacuous lookahead from a refusing one"
  - "is a rule named _sv_only actually gated"
tags: [profiles, dialects, lookahead, grammar, engine-universal, controls, sv-corpus-grad]
date: 2026-08-25
status: current
evidence: "PGEN-SV-CORPUS-GRAD-0303, sizing SV-CORPUS-GRAD.13e.10(c4). Measured on a 5-rule synthetic driven with ast_pipeline --interpret-parse --grammar-profile, with a proven-active control: a REQUIRED gated rule accepts under `loose` and REJECTS under `strict` (so the gate is on), while `!X` with a catch-all able to consume the guarded token REJECTS under `loose` and ACCEPTS under `strict`. --lint-grammar is completely clean on this: no profile_orphans, no unreachable_rules, no warning. Live population in PGEN's SystemVerilog grammar: 3 sites, all !scope_resolution, with scope_resolution gated to [sv_2017, sv_2023]; none produces an over-acceptance today because nothing else under verilog_2005 can consume `::`, so the parse fails on the unconsumable token regardless — the hazard bites only when some other alternative CAN consume what the lookahead guarded against. TWO EARLIER ATTEMPTS TO MEASURE THIS PRODUCED CONFIDENT NON-RESULTS: (1) @profiles written INLINE after := is a BRANCH-level gate and never gated the rule, revealed only by the control arm; (2) the corrected probe could not discriminate, because with nothing able to consume the token a vacuous lookahead and a refusing one both end in a reject."
reverify: "⛔ MEASURED ON THE INTERPRETER ONLY — the probe makes 0 generated-parser invocations, and TOOLBOX 1.5b marks the interpreter authoritative BY VERIFICATION, not by construction. Reproducing on the shipped engine (scratch slot 1.3 / compile_and_parse 1.4) is step (a) of ENGINE-UNIVERSAL-SERVICES.46 and is OWED. bash docs/tasks/artifacts/sv_corpus_grad/profile_gate_sizing/probe.sh   # 6/6, ~1s. Arm [1] is the control that proves the gate is active; arm [2] is the finding: !X rejects under loose and ACCEPTS under strict."
---

# A profile gate inverts every negative lookahead on the gated rule

**Question it answers:** I am about to gate a rule out of a dialect profile. What breaks that no lint
will tell me?

**Answer:** every `!X` in the grammar, where `X` is what you gated — and it breaks in the
*accepting* direction, in the profile you were trying to make *stricter*.

⛔⛔ **EVIDENCE BOUND, stated before the finding is used: this is measured on the INTERPRETER.**
The probe makes zero generated-parser invocations, and PGEN's own toolbox marks the interpreter
authoritative *by verification, not by construction*. Reproducing it on the shipped engine is
owed work, not a formality — if the generated parser does not reproduce it, this card is wrong.

## The mechanism

`!X` means *refuse if `X` matches here*. Gate `X` out of a profile and `X` matches nothing, so `!X`
succeeds **vacuously**. The guard evaporates exactly where you asked for more strictness.

| arm | `loose` (X live) | `strict` (X gated) |
|---|---|---|
| a **required** gated rule — ⭐ the control that proves the gate is on | accept | **reject** |
| `!X`, with a catch-all able to consume the guarded token | reject | ⛔ **accept** |

`--lint-grammar` is clean throughout: no `profile_orphans`, no `unreachable_rules`, no warning of any
kind. A positive-only test suite cannot see it either, because the symptom is an *extra* accept.

## When it actually bites

Only when **some other alternative can consume what the lookahead was guarding against**. Measured in
PGEN's SystemVerilog grammar: 3 live sites (all `!scope_resolution`, with `scope_resolution` gated to
the SV profiles) and **zero** over-acceptances today — `p::b` still rejects under `verilog_2005`,
because nothing there can consume `::`, so the parse dies on the unconsumable token anyway.

⇒ **undefended, not absent.** The population is one grammar edit away from being live, and the edit
that does it is a *strictness* fix.

## ⭐⭐ The two non-results that looked like results

Both of these ran, printed verdicts under two profiles, and measured nothing:

1. **The gate was never on.** `@profiles` written *inline* after `:=` is a **branch-level** gate; the
   rule-level form is a line of its own **above** the rule. Placement is semantic. The only thing
   that revealed it was the control arm — a *required* gated rule that still parsed under `strict`.
   Without that arm the probe would have published *"the hazard does not exist"*.
2. **The design could not discriminate.** With nothing able to consume the guarded token, a vacuous
   `!X` and a refusing `!X` **both end in a reject**. Two hypotheses, one reading — see
   [[feedback_a_control_that_passes_under_both_hypotheses_is_not_evidence]]. The fix is to add a
   catch-all that *can* consume it, so the hypotheses predict opposite verdicts.

⇒ **run the control arm first, and design the probe so your two hypotheses disagree.** A probe that
cannot distinguish them is not slow evidence, it is no evidence.

## What to do instead

- Gate the **consuming sites**, not the terminal, when the terminal is also referenced under a `!`.
- Before gating, grep for `!<rule>` and for `!( … <rule> … )`; the population is usually tiny and a
  blocking lint arm is affordable.
- ⚠️ And check the gate is real: **a naming convention is not an enforcement mechanism.** PGEN has
  rules named `…_sv_2017` that are `@profiles`-admitted to `verilog_2005` — the name says one thing
  and the annotation says another → [[a-rule-named-sv-only-is-not-thereby-gated]].

Sibling of [[two-counters-bound-each-other-only-if-they-count-the-same-population]] (both are
"the instrument was answering a different question than I asked").
