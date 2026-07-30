# feedback_routine_decisions_are_not_escalations

- **Category:** feedback (standing directive — director, 2026-07-31)
- **Status:** binding

## Context

`LANG-CAPABILITY-AUDIT.10.2` and `.10.6` closed, and the engineer ended the turn by asking
the director to approve `.10.3` — *"`.10.3` is now unblocked … Small, and it closes the
`EBNF-SOURCE-OF-TRUTH` breach you named in #208."* `.10.3` was already a chartered leaf, in
an active tree, whose only stated blocker had just been measured away, implementing a
directive the director themself had issued in session #208. Nothing about it was ambiguous,
outward-facing, or hard to reverse.

The director's reply: *"What do you want me to answer for [that]? If you need my greenlight,
you have it. The only [thing] I am asking is that all your decision be sota, signoff and
highly professional and top-notch."*

## Decision

> **A decision that is routine within the agreed principles is the engineer's to make
> silently. Escalating it is not caution — it is offloading judgement, and it costs the
> director a turn to hand back authority they had already delegated.**

The bar is not "did I have explicit permission for this exact leaf?" but **"is this SOTA,
signoff-quality, and professional?"** If yes, do it and report it.

## How to apply — the test before escalating

Escalate ONLY if at least one holds:

1. **Ambiguous direction** — two readings lead to materially different work, and measurement
   cannot settle which is wanted.
2. **Outward-facing or hard to reverse** — a published surface, a deletion, a push, anything
   spending someone else's budget.
3. **Scope change** — it opens a new direction, or contradicts a recorded decision.
4. **A cost only the director can price** — CI minutes, release timing, downstream breakage.

Otherwise: **decide, execute to the bar, and surface the RESULT.** A finding, a measured
surprise, or a corrected premise is always worth surfacing — that is directive #9 and it
remains in force. A *permission slip* for chartered work is not.

⚠️ The failure mode this corrects is subtle, because it wears the costume of diligence: the
escalation was phrased as courtesy, and it still stalled a leaf that was ready to run.

## Consequences

- `.10.3` and the Perl-tree removal (`.10.7`) both carry standing director approval; neither
  needs re-asking. Recorded in their leaves so a fresh session inherits the authority.
- Related: [[feedback_flow_findings_are_routed_not_worked]] (route findings, don't stall on
  them) and the surfacing directive's own boundary — *"routine implementation decisions
  within our agreed principles are yours to make silently. Only surface the non-routine."*
