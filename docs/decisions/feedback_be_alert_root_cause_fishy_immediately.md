---
name: feedback-be-alert-root-cause-fishy-immediately
description: STANDING, emphatic (director 2026-06-07) — BE ALERT and IMMEDIATELY root-cause + flag foundational / "fishy" / not-normal results yourself; NEVER classify-and-route a suspicious finding as "structural, follow-up" without explaining the mechanism. The director should NOT have to point out that a grammar-driven generator emitting parser-rejected output is abnormal — catching that is the agent's job.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-07
id: feedback-be-alert-root-cause-fishy-immediately
title: A fishy or foundational result is root-caused IMMEDIATELY — never classify-and-route it as 'structural, follow-up'
date: 2026-06-07
answers:
  - "a result looks surprising or foundational — do I flag it or keep going"
  - "can I route a suspicious finding as structural follow-up"
  - "who is responsible for noticing an abnormal result"
  - "the generator emitted output its own parser rejects — is that normal"
reverify: grep -n 'fishy' CLAUDE.md docs/decisions/feedback_be_alert_root_cause_fishy_immediately.md | head -4
---

**Director directive (2026-06-07), emphatic.** "I am surprised you do not take on yourself to understand
the root cause… that's not normal… you should have created a task-tree to investigate further, because it
is fishy." Then: "This should not happen again, I should not have to point this sort of thing out, that's
part of your job to be alert and immediately flag this sort of nasty issue."

**The rule.** When a result is FOUNDATIONALLY SURPRISING — it contradicts how the system *must* work
(e.g. a generator that derives strings from the same EBNF the parser uses is emitting strings the parser
REJECTS) — that is a red flag. STOP and ROOT-CAUSE it immediately, tools-first, to the actual mechanism;
create a task-tree to own it; and explain it plainly. Do NOT bucket it as "structural, route to a
follow-up" and move on. Being alert to "this shouldn't be possible" and chasing it down is the agent's
job — the director must not have to notice it for me.

**Why classify-and-route is a failure here.** "Classified structural, routed to G.4" gave a tidy-looking
label while leaving the real, abnormal question unanswered (HOW can grammar-driven generation produce
grammar-rejected output?). A label is not an explanation. Worse, the label was partly WRONG — I listed
`(?|)`/`(?P>)`/`(?(…))` as failing when they parse fine; only `\u` and `(*verb)` fail — because I
pattern-guessed from the samples instead of pinning each failure (cf. [[feedback_why_and_where_before_solution]]).
The actual mechanism (an out-of-band validator the generator can't see) is a real DEFECT class
([[project_ebnf_is_single_source_of_truth]]).

**How to apply.**
1. Treat "this contradicts an invariant / shouldn't be possible / is fishy" as a STOP-and-investigate
   trigger, surfaced proactively — not deferred until the director asks.
2. Root-cause to the mechanism with tools (here: `parseability_probe` for the exact error;
   `--trace high`/`--trace debug` to see what the generator is doing; `grep` to find the validator);
   never substitute a category label for the mechanism.
3. Open a task-tree to own a foundational finding; record the rule loud (book + KM + decision).
4. Report only what a tool actually showed; don't guess the failing construct from surrounding text.

Reinforces [[feedback_tools_first_no_guessing]], [[feedback_why_and_where_before_solution]],
[[feedback_understand_subsystem_holistically_first]], [[feedback_no_codebase_change_without_tool_backed_facts]],
[[feedback_always_signoff_decisions]].
