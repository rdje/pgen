# feedback — every issue, bug or misbehavior RAISED must be FIXED; routing is a schedule, never a disposal

**Category:** `feedback` (standing discipline — binds every session, every harness)
**Stated:** director, 2026-08-07, verbatim:
*"Each and every issues, bugs, misbehavior you raise or uncover have to be properly fix, you may not
just log them and forget about them, you may not do that. You need to address each one of them
either immediately or deferred."*

**Context:** given immediately after `PGEN-SV-EXH-PROOF-0178`, and *provoked by it.* The preceding
session surfaced three findings and, asked whether it had fixed them, answered honestly that it had
not: one candidate refuted, one property measured-but-not-structural, one artifact defect noted.
`-0178` then converted the last two into leaves **with designs**. The director's point is that this
was still not the deliverable. A leaf carrying a design is a schedule; the deliverable is the fix.

## The rule

1. **Anything you raise, you own.** If a session surfaces an issue — a bug, a misbehavior, a
   property that holds only by measurement, a non-reproducible artifact — it does not get to end as
   a record. It ends as a fix, or as a **dated, ordered, owning leaf that a later session actually
   executes.**
2. **"Deferred" is a schedule with a position, not a shelf.** A deferred finding must sit in a
   frontier table with an order number, so the next session cannot pick something else without
   consciously stepping over it. A finding parked with no position is indistinguishable from a
   finding forgotten.
3. **A finding whose fix is designed is HALF done, and must be reported as half done.** Do not let
   "designed and routed" read as "handled" in a completion message. Say which of the raised items
   are fixed and which are not, in those words.
4. **Refuted counts as addressed; unfixed does not.** A candidate measured and disproven (with its
   numbers recorded and the code reverted) is a legitimate terminal state for *that candidate* — but
   the underlying defect stays open and keeps its position in the queue.

## ⚠️ How this reconciles with [[feedback_flow_findings_are_routed_not_worked]]

They are not in tension; this one **closes that one's escape hatch.** That doctrine sets the
DEFAULT — a flow finding is routed rather than worked *right now*, so product work is not displaced
by whatever the session happened to trip over. It says nothing about the routed item ever being
done, and that silence is what this directive removes:

> routing decides **WHEN** a finding is worked. It never decides **WHETHER**.

⇒ a routed finding still has to be executed. The `routed, waiting behind product` list in `MEMORY.md`
is a **queue**, not an archive, and an entry that has sat there across many sessions is itself a
defect worth surfacing.

## What good looks like

- Fix it in-slice when it is small, safe and verifiable — a serialization determinism defect, a
  missing bound, an off-by-one in a control.
- When it is genuinely a separate unit of work: open the leaf, **write the design and the prior-art
  read into it** (so the next session implements rather than re-derives), give it a position in the
  frontier table, and say plainly in the completion message that it is unfixed.
- Never: surface it, admire it, and move on.

## Enforcement

Partly mechanical already — `WAIVER-ROUTING` fails a commit whose leaf claims a gate does not apply
without naming the leaf that owns fixing it, and `ROUTING-EVIDENCE` fails a finding routed out of a
tree without the measurement behind the routing. Neither checks that a routed leaf is ever
*executed*; that gap is the one this record governs by discipline until an instrument exists for it.
⇒ if a mechanizable check for "a leaf routed N sessions ago that nothing has picked up" can be
written, it belongs in `scripts/check_doctrines.sh` — see `DOCTRINE_ENFORCEMENT.md` §3.

**Related:** [[feedback_flow_findings_are_routed_not_worked]] (the WHEN),
[[feedback_be_alert_root_cause_fishy_immediately]] (never classify-and-move-on),
[[feedback_answer_your_own_technical_questions]] (a technical fix is yours to sequence, not to ask
about).
