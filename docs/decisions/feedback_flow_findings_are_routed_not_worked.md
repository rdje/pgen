# feedback — a flow finding is ROUTED by default, and WORKED only when it blocks product

**Category:** `feedback` (standing discipline — changes the DEFAULT for an entire class of work)
**Stated:** director, 2026-07-30, session #227 — verbatim, across three messages:
*"it looks like we might spend the rest of the project fighting with these flow issues than actually
doing real coding ? Can you fix those flow issues once and for all ?"* ·
*"It is good that you are uncovering all these issues though."* · *"WE need to have a clean flow."*

**Context:** given after a session that closed `CI-PARITY-GATE-ROT.7` — `sota_exit_gate` green
end-to-end for the first time — and surfaced five findings, four of which were not open flow work.

## ⛔ THE MEASUREMENT THAT SETTLES THE PREMISE — the director is right, decisively

Commits classified by what they **touched**, not by their titles
(`grammars/` + `rust/src/` + `generated/` = PRODUCT; `rust/scripts/` + `scripts/` + `.githooks/` +
`.github/` + Makefiles + `build.rs` = FLOW; everything else = DOCS):

| window | PRODUCT | FLOW | DOCS |
|---|---|---|---|
| last **60** commits | **1** | 27 | 32 |
| last **20** commits | **1** | 9 | 10 |

**One commit in sixty touched a grammar, the Rust pipeline, or a generated parser.** This is not an
impression to be managed; it is the shape of the work, and it is wrong.

## ⭐⭐ WHY THE PREVIOUS "ONCE AND FOR ALL" ORDER DID NOT HOLD

This is the **second** time this order has been given. Session #218 (2026-07-28), verbatim:
*"resolve all these flow related issue in a clean, sota and signoff, once and for all"* + *"so that we
can come back to real coding activities!"*. It was executed in the ordered scope and flow work still
dominated every session after it.

The reason is structural, not a failure of will: **the flow backlog was self-generating and its size
was unknowable.** `sota_exit_gate` had never completed a run, so it revealed exactly one blocker per
fix — six of them, each hidden behind the last. The record from that session says so plainly:
*"nobody knows how many remain."* ⇒ **an order to finish an unbounded list cannot be satisfied**, so
each session honestly closed the next blocker and the list refilled.

**What changed on 2026-07-30:** the aggregate is green end-to-end (`CI-PARITY-GATE-ROT.7`,
`PGEN-CI-PARITY-GATE-ROT-0024`). The generator of new flow findings is off, and the remainder is
**visible and finite for the first time** — `CI-PARITY-GATE-ROT.10`/`.11`/`.16`/`.17`,
`OPS-MEMSAFE.4`, `DOCTRINE-GAP-OWNERSHIP.3`, `DONE-BAR.1b`. Seven priced items, none blocking.

## ⭐⭐⭐ THE RULE — and it is NOT a weakening of the no-slide directive

A standing directive already governs findings (session #217, verbatim):
*"you shouldn't let any issue, even the smallest slide … do whatever it takes to be 100% sure to
address it either now or later but do not let it slide unaddressed."*

⭐ **Read it exactly: "now OR LATER".** The directive forbids **dropping** a finding; it does not
require **fixing it immediately**. We have been reading *"do not let it slide"* as *"fix it now"*, and
that reading — combined with genuinely rigorous measurement — is what produced 1 product commit in 60.
**The task-tree IS the "later."** Routing a finding to a tracked leaf, with its evidence, satisfies the
directive in full.

Therefore, the default for a flow/ops finding:

1. **ALWAYS route it.** A leaf, in the owning tree, with the measurement that found it. Never a
   mention in a message, never a TODO, never nothing. (`DOCTRINE-GAP-OWNERSHIP` exists because a
   recorded-but-untracked fact sat 58 commits.)
2. **WORK it now only if it BLOCKS**: a product gate cannot run, a proof surface cannot be trusted, or
   a published claim is false. Otherwise it waits its turn behind product work.
3. **Fix in place, without a leaf, only what is trivial and adjacent** — the session-#218 constraint
   (*"every finding met on the way was fixed IN PLACE"*) still applies to genuinely small things met
   en route. The judgement call is size, and it is the engineer's.
4. **Report flow findings as a bounded list, not as a headline.** A findings callout that presents one
   flow bug alongside four non-issues reads as a five-item backlog and misinforms the director about
   where the project stands. That happened in session #227 and is the proximate cause of this record.

## What "a clean flow" means operationally (director: *"WE need to have a clean flow"*)

Clean is **not** "zero known findings" — that is the unbounded reading that failed twice. Clean is:

- every proof surface a product claim rests on can RUN and its verdict can be TRUSTED (the aggregate
  green, the doctrines passing, no instrument reporting a number it cannot defend);
- every known defect is ROUTED and visible;
- and the flow does not consume the sessions.

⇒ the flow is clean **when flow work is no longer the thing we do**, not when the flow backlog is
empty.

## Consequences

- The seven remaining flow items are worked as a **bounded batch or behind product**, never as the
  default activity of a session.
- The next substantive work is **product**: the parser families and the EBNF capability program
  (`LANG-CAPABILITY-AUDIT.4` — which the director already ranked *"the highest-value open leaf, AHEAD
  of `.6`"* in session #209, a ruling that has been outstanding while hygiene ran).
- ⚠️ **Honest limit:** this record changes a default, and a default is a judgement, not a gate. Nothing
  mechanically prevents a future session from spending itself on flow again. The measurable tripwire is
  the table at the top of this record — **re-run that classification when a session feels
  flow-heavy**, and if PRODUCT is still ~1 in 60, the discipline is not holding.

## Related

- `docs/decisions/feedback_delete_reclaimable_artifacts_regularly.md` — same shape: a standing
  authorization that removes a per-request ask while keeping the proof obligation.
- `docs/tasks/CI-PARITY-GATE-ROT.md` `.7` — the closure that made the remainder finite.
- `docs/tasks/DOCTRINE-GAP-OWNERSHIP.md` — why routing is mandatory rather than optional.
