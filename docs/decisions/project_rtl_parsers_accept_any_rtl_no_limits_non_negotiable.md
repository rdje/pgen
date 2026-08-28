---
name: project_rtl_parsers_accept_any_rtl_no_limits_non_negotiable
description: STANDING CAPABILITY MANDATE (non-negotiable) — every PGEN RTL parser must ingest ANY RTL file, hand-written or generated, optimized or not, with NO limitation or restriction. A fixed acceptance ceiling is a defect, not a bound to be justified.
metadata:
  type: project
---

**Director ruling, 2026-08-28**, given while `ENGINE-UNIVERSAL-SERVICES.63` was mid-flight
re-adjudicating the 4096-frame recursion ceiling:

> *"PGEN SV and VHDL or any of the RTL related parser shall be able to parser, ingest any type of RTL
> files we throw at them, no limitation or restriction there, hand-written, generated, optimized or
> not, anything, they should be able to parse them, no exception, non-negotiable"*

and, refining it in the same exchange:

> *"The only obvious limit shall be the host RAM and disk allowance, but the PGEN parsers shall be
> thorough and accurate and fast (not reached yet, because it is not the focus right now)."*

## What it decides

⭐⭐ **The second sentence is the CONSTRUCTIVE half and it names the acceptable bound.** *"No limit"*
does not mean *unbounded* — it means **the bound must be the HOST's resources, not a compiled-in
constant.** A parser may run out of the machine's RAM or disk; it may not run out of a number a
developer typed. That is a testable design rule, not a slogan:

- ⛔ **A constant that bounds acceptance is illegitimate** regardless of how it is justified — depth,
  iteration count, size, node count. It refuses input the host could have handled.
- ✅ **A bound derived from real, measured host resources is legitimate**, and is what the guards must
  become: capacity sized from actual stack/heap headroom, discovered at runtime, so a bigger machine
  parses a bigger file with no code change.
- ⇒ the acceptance question *"will this file parse?"* must be answerable as *"does this host have the
  memory for it?"* and by nothing else.

⚠️ **And the ordering is restated in the same sentence**: *thorough and accurate* first, *fast* last,
with speed explicitly *"not the focus right now"* — consistent with [[feedback_correctness_before_speed]].
A capacity fix may cost throughput; that is an accepted trade under this ruling, not a regression to
be argued about. It may **not** cost accuracy.

It converts an open engineering question into a **settled requirement**. The question
`ENGINE-UNIVERSAL-SERVICES.63`(c) was chartered to answer — *is the ceiling's location defensible, and
what are the alternatives worth?* — is answered from above: **there is no defensible location for a
fixed acceptance ceiling.**

⛔ **A parser limit that turns conforming RTL into a rejection is a DEFECT.** Not a documented
capacity, not an acceptable engineering trade, not a "declared bound". The words *no limitation or
restriction*, *no exception* and *non-negotiable* leave no room for a graded reading, and they were
given after the leaf had already reported that the limits are reachable from real vendored input.

## Scope — every RTL family, every limit, not one constant

- **Families**: SystemVerilog, VHDL, *"any of the RTL related parser"* — so the mandate is
  engine-universal, not an SV special case, and a fix must be made in the engine.
- **Input classes**: hand-written, generated, optimized, *"anything"* — the same four classes
  [[project_nexsim_target_population_is_all_rtl_assume_the_worst]] named, now with the pessimistic
  reading made mandatory rather than merely assumed.
- **Limits**: this is a rule about the CLASS, not about the one constant that provoked it. Every
  mechanism that can refuse conforming input for a reason that is not "this input is not in the
  language" is in scope. `.63` had found **two** before the ruling landed:
  1. `GENERATED_RECURSION_GUARD_MAX_DEPTH = 4096` (`ast_pipeline/ast_based_generator.rs:31`,
     mirrored in `ast_based_generator/cascade.rs:648`) — the recursion ceiling. Measured worst-shape
     capacity is **61 source levels**, not the 249 `.61`(b) published.
  2. `SAFETY_LIMIT = 10_000` (`ast_based_generator.rs:6587`, `ast_based_generator/cascade.rs:1704`) —
     a per-quantifier iteration cap that does not even refuse: it `break`s, silently truncating the
     repetition, so the parse fails at the enclosing level with an ordinary "did not consume full
     input". The protocol path logs a warning only under `trace_enabled()`; the fused path logs
     nothing at all.
  ⇒ **before declaring the mandate met, census the engine for the whole class** — any other constant
  bounding depth, iteration, size or count on the accept path.

## What it does NOT license

⛔ **It is not a licence to remove the guards and let the process die.** A stack overflow is a worse
outcome than a diagnosed rejection: it aborts the host, and Nexsim is an embedder. The requirement is
that **conforming input parses**, which is a statement about the accept path. The engine still owes a
bounded, non-crashing answer on input that is genuinely not in the language, and on genuinely
unbounded recursion the guard is what makes that possible. ⇒ per the *"host RAM and disk"* refinement,
the work is to **re-anchor each guard on measured host headroom** — grow the stack and bound by real
remaining stack rather than by a frame count; let a quantifier iterate while memory allows — **not to
delete the guard**. The guard survives; its threshold stops being a literal.

⛔ **It does not re-open the analog/AMS scope.** [[project_nexsim_target_population_is_all_rtl_assume_the_worst]]
records the director's same-day scoping of Verilog-AMS as a HORIZON goal behind the first official SV
release (`ENGINE-UNIVERSAL-SERVICES.64`, parked). *"Any type of RTL"* here binds the families that
ship today.

## The evidence that was already on the table when it was given

Measured in `.63`(a)/(b) on the shipped parsers, before the ruling:

- The **worst emitter-plausible shape caps at 61 source levels** (`$signed({1'b0, (s ? a : (…))})`,
  67.1 frames/level) — a 4.1× reduction on `.61`(b)'s published worst of 249, whose own honest bound
  (i) had said *"assume a worse shape exists until the search says otherwise"*.
- A **real vendored file already carries a 256-level conditional chain**
  (`stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh330.v`, a truth table lowered to a mux chain;
  the parser's own AST depth on it is **561**). `.61`(b)'s bracket-depth oracle reads it as shallow,
  which is why that slice published *"deepest real nesting = 22"* and an *"11.3× margin"*.
- A **real vendored machine-generated file's own 568-item `unique case`**
  (`opentitan/hw/top_earlgrey/ip_autogen/pinmux/rtl/pinmux_reg_top.sv`), lowered by the canonical
  case→mux transformation with parentheses, **is REJECTED** by the shipped SV parser, ceiling-labelled.

⇒ the mandate did not overrule a measurement. It agreed with one.

Related: [[project_nexsim_target_population_is_all_rtl_assume_the_worst]],
[[project_nexsim_sv_signoff_delivery_focus]], [[feedback_correctness_before_speed]],
[[an-honest-bound-that-names-a-population-exclusion-is-a-ruling-waiting-to-be-overturned]].
