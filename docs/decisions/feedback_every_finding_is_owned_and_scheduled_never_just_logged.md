---
name: feedback_every_finding_is_owned_and_scheduled_never_just_logged
description: "STANDING DIRECTIVE (director 2026-08-10, session #232, stated three times in one turn): every issue raised must be LOGGED, TASK-TREE OWNED, TRACKED, and FIXED — either now, or explicitly postponed/deferred to a named later point. Never logged and forgotten. Surfacing a finding is the FIRST step, not the deliverable; a finding parked without a scheduled fix will bite harder later, and is neither SOTA nor sign-off nor professional."
metadata:
  node_type: memory
  type: feedback
---

## The directive (director 2026-08-10, session #232 — verbatim, across three messages)

> *"Did address the 3 issues you raised … in order to fix them, if they needed fixing?"*

> *"Ok, what's the point in logging issues if they are not fixed or deferred to be fixed later?
> What's the point. If you do not fix them they will bite you later even harder?"*

> *"Logging issues is the first step them, but not making sure they will be fixed now or later is a
> real concern."*

> *"So, I do not understand the rational in simply logging them and then forget about them, what's
> the ultimate goal here. It is not sota, it is not signoff and it is not how professional would do
> things, so why are you doing it?"*

> **"Everything issue you raise must be logged, task-tree owned, tracked and fixed either now or
> postpone, deferred for later when the time is right. Never logged and forgotten."**

## The rule, operationally

A finding is not handled until **all four** hold. Three of four is a graveyard entry.

| # | step | what counts | what does NOT count |
|---|---|---|---|
| 1 | **LOGGED** | written to a durable layer with its measurement | a sentence in the chat |
| 2 | **OWNED** | a named task-tree leaf exists and the finding is *inside* it | "routed to tree X" with no leaf |
| 3 | **TRACKED** | the leaf is reachable from `docs/TASK_TREE.md` / `MEMORY.md` | buried mid-file with no frontier pointer |
| 4 | **SCHEDULED** | fixed NOW, or deferred with a **named trigger** — "when the strictness switch design opens", "after the SV release" | `todo`, no date, no trigger, no priority |

⭐ **Step 4 is the one that was missing.** This project already says *"every finding is FIXED —
routing decides WHEN"* in its north star; the failure mode is treating **routing as a substitute for
deciding when**, so `todo` becomes a synonym for *never*.

## The specific error that produced this directive

Session #232 surfaced three findings while closing `SV-CORPUS-GRAD.3.26`/`.3.27`, wrote all three
into task leaves, and fixed **none** of them — then reported them under a "🔎 Findings for your
review" banner as if surfacing were the deliverable.

⛔ **The self-justification was the interesting part, and it was wrong.** The finding that mattered
(a 12 GB unbounded-memory divergence on valid industry RTL, `SV-CORPUS-GRAD.11a`) was deferred by
applying the **SV lane lock** — *"a finding in another family is ROUTED to a parked leaf, never
worked"*. But that rule exists to stop the session **leaving** the SV lane, and this finding is
**inside** it. A focus-protection rule was used as a deferral excuse for in-lane work that could have
been started immediately. ⇒ **check which side of the lane a finding is on before routing it**; the
lock never licenses deferring the lane's own defects.

## How to apply it

- **Default to fixing now** when the finding is in the active lane and the fix is bounded. Routing is
  for work that is genuinely out of lane or genuinely large.
- **When deferring, write the trigger, not a status.** *"`todo`"* is not a schedule.
  *"blocked until the strictness switch design opens (`LRM-GRAMMAR-FIDELITY.1c`)"* is.
- **When a finding is large, split it**: do the bounded diagnostic slice NOW (reproducer, root cause,
  growth curve) and schedule the engineering. A leaf holding a minimal reproducer is tractable; a
  leaf holding a sentence is not.
- **Report honestly.** If a finding is surfaced and not fixed, say *"not fixed, here is when"* rather
  than presenting the log entry as the outcome.

## Related

- [[project_standing_tripwires]] — the live traps register; same discipline, different axis.
- `DOCTRINE-GAP-OWNERSHIP` (task tree) — the sibling concern the director raised on 2026-07-27:
  *"a known defect written into a decision record is NOT tracked work"*. **This directive is that one
  generalized**: it is not enough for a defect to be *written down and owned*; it must also be
  *scheduled*.
- [[feedback_prefer_feature_work_over_governance_lanes]] — ⚠️ read together with this: prefer product
  work, but "prefer feature work" is not a licence to leave in-lane defects unscheduled. The two
  compose as *fix the lane's own defects; park other lanes' findings with a trigger*.
