---
name: feedback_say_what_i_did_what_i_did_not_and_what_i_need
description: "STANDING DIRECTIVE (director, 2026-08-20, session #250) — 'I'll say plainly what I did, what I didn't, and what I actually need. No more dressing.' Adopted as a RULE, not an intention, after three instances in one session where unfinished work was reported as if it were a property of the problem: 'GENERATED-REPRODUCIBILITY still needs a human' (it needed the work doing — nothing there required judgement), 'a recurring cost, stated rather than discovered' (a choice I made, framed as a fact about the design), and 'routed with the measurement' (jargon whose plain meaning is 'I did not fix it'). Each was accurate word-by-word and misleading as a whole, because the framing moved authorship of the gap from me to the problem. The director's verdict on the pattern: 'Your claims or requests for call are really confusing sometimes.'"
id: feedback_say_what_i_did_what_i_did_not_and_what_i_need
title: "Say plainly what I did, what I did not, and what I actually need — an unfinished item is never a property of the problem"
date: 2026-08-20
evidence: "docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .39 (the 'still needs a human' retraction — tier 2 proved the tree correct and left the doctrine RED; nothing in it required judgement, and it was closed in one slice once stated plainly); leaf .38 (the 'recurring cost, stated rather than discovered' that the director refused, then measured as BLOCKING EVERY COMMIT — bigger than the version I published); docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2x.4."
reverify: "grep -rl 'needs a human\\|still needs a human' docs/tasks/*.md | head   # any surviving instance must name what judgement is actually required, or be work I have not done"
answers:
  - "how should I report work I have not finished"
  - "when is something genuinely a decision for the director"
  - "my status update was called confusing — what was wrong with it"
  - "is it enough to document a cost honestly instead of fixing it"
  - "how do I phrase a routed or deferred item"
  - "what counts as dressing in a status report"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-20
---

**The rule.** Every report says three things and separates them: **what I did**, **what I did not
do**, and **what I actually need from the director**. Nothing else gets to occupy those slots.

**Why it became a rule.** Three times in one session I described my own unfinished work as though it
were a characteristic of the problem:

| what I wrote | what it meant |
|---|---|
| *"GENERATED-REPRODUCIBILITY still needs a human"* | I had not automated it. Nothing in it needed judgement. |
| *"a recurring cost, stated rather than discovered"* | I chose not to fix it, and named the choice like a finding. |
| *"routed with the measurement"* | I did not fix it. |

Every one is defensible word by word. Together they shift authorship of the gap from me to the
problem, and they cost the director real attention — being asked to weigh something that was never a
decision. **Transparency about a defect is not a substitute for removing it**, and phrasing that
makes an unfinished item sound structural is worse than silence, because it also discourages anyone
from trying.

⭐ **The test before writing "needs a human", "requires a decision", or "your call":** name the
judgement. If I can state the competing options and why the choice is not derivable, it is a real
escalation. If I cannot — if the honest sentence is *"this is derivable and I have not done it"* —
then it is my work, and saying so plainly is the whole obligation.

⛔ **This does not license hiding a gap.** What I did not do still gets stated, explicitly and
without softening, along with what it costs and what would close it. The change is that it is
labelled as **mine and unfinished**, not as a property of the terrain.

⚠️ **And it applies to costs as much as to tasks.** *"This has a recurring cost"* invites acceptance.
*"I have not removed this cost yet, and here is what removing it takes"* invites the correct
question, which the director asked twice in one session and was right both times.

See also [[feedback_answer_your_own_technical_questions]] and
[[feedback_verify_a_claim_three_ways_before_publishing_it]].
