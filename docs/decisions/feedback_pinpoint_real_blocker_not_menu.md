---
name: feedback-pinpoint-real-blocker-not-menu
description: STANDING, emphatic (director 2026-06-22) — PINPOINT the real blocker tools-first; do NOT offer strategy menus or "try things". For a hard problem, prove the EXACT mechanism + source location with the debug toolbox, then propose ONE clean SOTA/robust/elegant fix to apply — not 2-4 options for the director to pick. General/parser-agnostic semantic annotations are PRE-AUTHORIZED as a fix vehicle when tool-proven needed.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-06-22
---

**Director directive (2026-06-22), emphatic.** "It looks like you are not able to clearly understand
the issues. You seem to be trying out things without being to able to pinpoint the real blockers."
And: "I do not want 4 strategies, I want you to apply one that works and is clean and SOTA and robust
and elegant. I do[n't] want 4 for me [to] pick, but one that [is] clean [and] works. stop talking,
just do the work." And: "If you need to add new semantic annotations that are general do it."

**The rule.** For a residual / hard problem: run the debug toolbox (see
[[feedback_systematically_use_debug_toolbox]]) until I can point at the EXACT blocking mechanism AND
its source location, THEN apply ONE minimal, clean, SOTA fix. Do NOT present a multi-option strategy
menu via `AskUserQuestion` — that reads as "I don't understand the blocker, you decide." The director
makes strategy calls only when there is a genuine real-world side effect; a technical root-cause →
fix is the agent's job to determine and execute.

**Why this was triggered.** On the SV cert `UNKNOWN` tail I offered a 4-way `AskUserQuestion`
("fix measurement / build STORE-AWARE-GEN / keep grinding / pivot"). The director read it as not
understanding the blockers. The correct move was already available: the debug-probe + semantic trace
PROVE the dominant blocker is a store-gate rejection (forced witness samples use undeclared
identifiers, so `has_fact`/`fact_attribute_equals` gates reject) → the one clean fix is store-aware
witness generation (declare-then-use), the `STORE-AWARE-GEN` lane.

**General annotations are authorized.** When the proven-minimal fix is a new GENERAL /
parser-agnostic semantic annotation (composing with [[feedback_features_parser_agnostic_enable_all_parsers]]
and [[project_vision_and_discipline]]), build it — do not park it or reach for a fragile hack.

Reinforces [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
[[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_user_is_director_not_engineer]],
[[feedback_be_alert_root_cause_fishy_immediately]], and the companion
[[feedback_systematically_use_debug_toolbox]].
