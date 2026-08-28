---
name: project_nexsim_target_population_is_all_rtl_assume_the_worst
description: STANDING POPULATION RULING — Nexsim's real flow includes ALL types of RTL (generated or not, optimized or not); assume the worst, across ASIC, SoC, digital and ANALOG projects. A parser claim scoped to hand-written open-source corpora does not discharge a bar.
metadata:
  type: project
---

**Director ruling, 2026-08-28**, given in direct answer to the open question `ENGINE-UNIVERSAL-SERVICES.61`(b)
raised to them (and overlapping `.57`):

> *"Assume NEXSIM real flow includes all types of RTL, generated or not, optimized or not, assume the worst
> here, but NEXSIM aimed at being used on real RTL, ASIC, SoC, digital chip and analog projects."*

## What it decides

**The TARGET POPULATION for every PGEN parser claim is all RTL a real chip project contains** — not the
population that happens to be vendored under `stimuli/*/subs/`. Concretely it includes:

- **machine-GENERATED RTL** — HLS output, register-map and interconnect generators, elaborated or
  flattened netlists, generator-emitted packages;
- **OPTIMIZED / tool-rewritten RTL** — the shapes synthesis and rewriting produce, which are not the
  shapes a human writes;
- **ASIC, SoC and digital chip** design and verification sources;
- **ANALOG projects**.

⛔ **"Assume the worst" is part of the ruling, not a stylistic flourish.** Where the evidence does not
cover a shape, the bar is the pessimistic reading, not the convenient one.

## The rule it creates

⛔⛔ **A measurement over the vendored open-source corpora BOUNDS NOTHING about the target population by
itself.** Those corpora are hand- and tool-written open-source RTL from fourteen upstream projects; they
are a *sample of one kind* of the population above. A claim of the form *"0 of N corpus files exhibit X,
therefore X is not a problem"* is **scoped to that sample and does not discharge a bar** — it must either
say so explicitly or be re-derived over the real population.

⇒ every future ruling that rests on corpus absence must name this record and state which of the four
classes above its evidence actually covers.

## What it immediately invalidated

`ENGINE-UNIVERSAL-SERVICES.61`(b) ruled the 4096-frame recursion ceiling's location *"defensible — do not
move it"* on exactly this reasoning: **0 of 30 056 corpus files contact it**. That measurement stands as a
fact and is not retracted. Its **RULING does not survive this record** and was REOPENED the same day
(`.63`): the same slice's own shape table shows `int'()` casts cliff at **249** nesting levels, ternaries
at **272** and concatenations at **314**, and deeply nested ternary mux trees and wide concatenations are
exactly what generated and optimized RTL emits. Under "assume the worst" the ceiling is a live risk, not a
theoretical one.

## Analog — a second, separate consequence

The ruling names **analog projects**. A fast census of `grammars/systemverilog.ebnf` finds **no
Verilog-AMS surface** (no `analog` block, `wreal`, `electrical`, `discipline`, `nature`). Whether Nexsim
needs AMS parsing — and in which language — is a CAPABILITY question this record raises and does not
answer; it is routed to `ENGINE-UNIVERSAL-SERVICES.64` and belongs to the horizon register
`LANG-CAPABILITY-AUDIT.11`. ⚠️ The grep is a fast negative, not an adjudication.

Related: [[project_nexsim_sv_signoff_delivery_focus]], [[project_horizon_universal_parser]],
[[feedback_correctness_before_speed]], [[an-x-is-checked-by-nothing-claim-is-a-census-claim]].
