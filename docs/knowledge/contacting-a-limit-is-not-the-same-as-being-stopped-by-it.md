---
id: contacting-a-limit-is-not-the-same-as-being-stopped-by-it
title: A run that SUCCEEDS can already have hit the limit and backed off — a pass/fail oracle is blind to the near-miss, and the near-miss cost 16.9x
answers:
  - "my parse passed, so was the input comfortably within capacity"
  - "why did the same input get dramatically slower when I shrank a resource reservation, without changing any verdict"
  - "why does my pass/fail suite not notice a large performance regression"
  - "how do I tell a hang from a merely slow run"
  - "is a timeout evidence that a program does not terminate"
  - "what should a resource guard report besides yes or no"
  - "why is a boolean the wrong return type for a guard that refuses for several different reasons"
tags: [instrument-soundness, observability, resource-limits, recursion, performance, claim-verification, engine]
date: 2026-08-30
status: current
evidence: docs/tasks/artifacts/engine_universal_services/eus69_stack_probe/ (readings, proportionality control, pre-registration); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .69(d) (PGEN-ENGINE-UNIVERSAL-SERVICES-0138); rust/src/dedicated_parse_stack.rs (StackProbeRefusal, StackProbeObservation); rust/src/ast_pipeline/mod.rs (RecursionGuard::extend_depth_probe); TOOLBOX.md section 3.3d
reverify: "python3 docs/tasks/artifacts/engine_universal_services/eus69_stack_probe/gen_inputs.py preserved_probes/eus69 && PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES=100663296 ./rust/target/release/parseability_probe --parse systemverilog preserved_probes/eus69/deep_3000.sv --profile sv_2017   # rc=0 (ACCEPTS) yet reports cause=BudgetExhausted; the same file at the 256 MiB default reports no_refusal and runs ~17x faster"
---

A guard that answers **yes / no** teaches everyone downstream to read its `no` as the interesting
case. Both halves of that habit are wrong.

## 1. The verdict is unchanged, the cost is not

PGEN's recursion guard refuses to descend further when the parse thread's stack budget is spent.
A refusal is *not* a rejection: the enclosing alternatives back-track and may still find an
accepting derivation. Measured on the shipped SystemVerilog parser, one 3 000-deep nested-paren
input, two stack reservations, **the same `rc=0` and the same AST**:

| reservation | verdict | wall clock | ceiling contacted |
|---|---|---|---|
| 256 MiB | accept | 9.67 s | no |
| 96 MiB | accept | 162.96 s | yes |

**16.9x, invisible to every pass/fail oracle in the repository.** Nothing in a corpus run, a
regression suite or a CI gate keyed on verdicts would report it, because no verdict moved.

⇒ **A green result is not evidence of headroom.** If a system has a resource ceiling, "did it
pass" and "how close did it come" are different questions, and only the second one predicts what
happens on a slightly bigger input or a slightly smaller host.

## 2. A timeout is a statement about the timeout

The same measurement was first read WRONG, and the error is worth more than the fix. At a 60 s and
then a 90 s bound the slow arm looked like a hang, and the finding was written up as *"never
answers"*. A 900 s re-run returned **`rc=0` in 162.96 s**.

Those two readings are not degrees of the same defect — they are different defects with different
owners. "Does not terminate" routes to the post-refusal re-exploration logic; "terminates at 17x"
routes to how the limit is sized. Publishing the first would have sent the next reader to the
wrong code.

⇒ **Before calling anything a hang, raise the bound until it either answers or you can say what
bound you actually proved.** A timeout bounds your patience; it never bounds the program.

## 3. One `false` for five reasons is not an answer

The guard refused for five distinguishable reasons — four of them statements about what the
process could not *learn* (no known stack extent, unknown growth direction, a per-frame cost too
small to divide) and exactly one about the resource really being spent. All five returned the same
`false`, and the frame count the rejection reported was identical in all five. So *"this input is
too deep for this host"* and *"this ran on a thread whose stack size we never knew"* were the same
observation from outside.

⇒ **A guard should report the cause and the quantity, not just the decision** — and the quantity
must distinguish *not measured* from *measured as zero*, because a guard that refuses at the first
thing it cannot establish never computed the later fields at all. Recording it on the refusal arms
of an already-`#[cold]` path costs the success path nothing, which removes the usual objection.

## The generalisation

Any guard, quota, retry budget, rate limiter or capacity check has a near-miss region where the
system still succeeds and pays for it. If the only instrument is the verdict, that region is
invisible, and it is exactly the region a capacity plan needs. Instrument the *approach* to the
limit, not only the crossing of it.

Related: [[an-instrument-can-be-green-in-both-arms-of-the-defect-it-describes]],
[[a-rising-pass-rate-is-not-evidence-of-correctness]],
[[a-reported-limit-without-its-provenance-invites-the-wrong-fix]].
