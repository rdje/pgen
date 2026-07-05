---
name: feedback-surface-insights-and-suggestions-explicitly
description: Director directive (2026-07-05) — when I surface a genuinely NOVEL idea / insight / suggestion / strategic observation that could open a new direction, change scope, or is worth the director's judgment (NOT a routine within-principle implementation decision), I SHALL raise it CLEARLY and PROMINENTLY for the director's explicit consideration and feedback — never bury it as a small comment in a long chat where it can be overlooked. Teamwork: the director must clearly hear it, think about it, and respond.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-07-05
---

**Director directive (2026-07-05).** *"Next time you have good ideas like this general grammar-AST
interpreter, don't just leave it as a small comment in the chat, I could have missed it and wouldn't
have asked question to clarify it, instead make sure I clearly hear about it, think about it and
provide my feedback explicitly … Now that I asked a few questions about this we have decided to
implement 3 task-trees so these sort of remarks, comments or suggestions from you shall be raised
clear to my attention. That's team work. There is a lot of message in this chat, we make sure I do not
overlook some of your insightful comments."*

**Trigger example (what went wrong).** The general grammar-AST interpreter was surfaced only as buried
prose ("the harness could be one of …") deep in a long turn. The director nearly missed it, had to ask
clarifying questions, and *then* we decided to build all three approaches as task-trees
(`PARSE-HARNESS`). Had it stayed buried, a valuable direction would have been lost in chat scrollback.

**What it means / how to apply.**
- When I produce a genuinely **novel idea / insight / suggestion / strategic observation** — something
  that could open a new direction, change scope, expose a latent risk, or is otherwise worth the
  director's judgment (e.g. *"we could build capability X"*, *"this hard gate might ALSO be unsound"*,
  *"there's a cleaner architecture here"*) — I **SURFACE IT PROMINENTLY**, not as an aside:
  - a clearly-labeled callout (e.g. a **`💡 Suggestion / for your call`** section, visually separated,
    at the END of the turn where it won't scroll away), or
  - for a genuine either/or the director should decide, the **AskUserQuestion** tool.
  - Make it **impossible to overlook** in a long chat, and invite explicit feedback.
- Also route the insight to a **durable layer** (a decision record / the resume pointer / a tree), per
  the memory doctrine — so even if a message is missed, the idea is not lost.

**Boundary — reconcile with the existing rules (NOT a contradiction, complementary).**
- [[feedback_user_is_director_not_engineer]] ("MAKE the technical decisions within agreed principles;
  don't manufacture decision points") and [[feedback_pinpoint_real_blocker_not_menu]] ("no 2–4 option
  menus") govern **ROUTINE, within-principle implementation decisions** — those I still just **make**.
- **THIS** directive governs **NON-ROUTINE, novel ideas / insights / direction-openers / latent-risk
  observations** — those I **raise explicitly** for the director's feedback.
- The distinction is the **class** of the thing: an implementation detail inside agreed principles is
  mine to decide silently; a *new capability, a strategic shift, a soundness/architecture insight, or a
  risk the director would want to weigh* is surfaced prominently. When unsure which class it is, lean
  toward surfacing it clearly (cheap; the cost of a missed good idea is high).

**It's teamwork.** The director wants to hear the good ideas and weigh in. Don't let an insight die in
chat scrollback. Reinforces [[feedback_always_signoff_decisions]] (surface, don't over-claim) and the
memory doctrine (route it to a layer, don't leave it only in the conversation).
