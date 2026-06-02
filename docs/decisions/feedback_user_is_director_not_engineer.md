<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_user_is_director_not_engineer.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_user_is_director_not_engineer
description: User-set 2026-05-25 (explicit role clarification) — user is NOT a software engineer; do not offload technical multiple-choice decisions to them. Make those decisions within agreed principles. Ask only for things with real consequences they need to weigh in on. Translate to plain language.
metadata:
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set (2026-05-25):**

> "You talk about direction, but I am not a SW engineer, hence I do not always fully understand the full consequence of the choices you providing me."

Triggered when I had presented multi-option AskUserQuestion choices like "apply hook? / continue defect? / end here?" — each option phrased in technical terms (level 5 vs level 1, predicate gates, engine vs grammar). The user has been making this point implicitly across the whole arc of session feedback ("just stick to what we agreed," "the meta is exhausting") and finally named it explicitly.

**The role split (durable):**

| Their role (director / principal)                                  | My role (engineer / technical lead)                            |
| ------------------------------------------------------------------ | --------------------------------------------------------------- |
| Set vision (e.g., SOTA SV parser for UVM corpus)                   | Translate vision into the technical work that achieves it     |
| Set principles (fix hierarchy, tools-first, targeted, no workarounds) | Make every technical decision WITHIN those principles         |
| Catch when I drift from the principles                             | Self-correct + report honestly when caught                    |
| Authorize things with real-world side effects                      | Recognize what needs authorization vs what is mine to decide  |
| Tell me when to stop / pivot / wrap                                | Execute until told otherwise; don't manufacture decision points |

**What requires user authorization (ask) — short, finite list:**
- `git push` (or any operation publishing to a public remote)
- Editing `.claude/settings.json` (per [[feedback_hook_scope]])
- Sending content to external services (LLMs other than ours, web posts, email, chat APIs)
- Deleting or overwriting content I didn't create or that contradicts how it was described
- Anything billed externally (mutants, cloud-review like `/ultrareview`)

**What is MINE to decide (do not ask):**
- Which technical approach to use within agreed principles — pick the one most consistent with grammar-first / tools-first / targeted / stick-to-what-works
- Which file to edit, which test to run, which trace to capture
- Whether to commit (commits don't publish — push does)
- Whether to write tests, instrument code, regenerate parsers
- Which slice ordering to use, which leaf-task to claim
- When to update / save memory
- When to update / read CLAUDE.md / docs / books
- Whether to investigate further before proposing — when in doubt, investigate more

**How to ask, when asking is genuinely required:**
- ONE question with a SHORT plain-language consequence-explanation, not technical alternatives
- Yes/no when possible, not multi-option
- State the consequence in user-facing terms ("this will email three reviewers," "this writes to a shared config that affects future sessions"), not in implementation terms

**Anti-pattern to avoid:**
- AskUserQuestion with 3-4 options described in jargon (level 5 engine vs level 1 grammar; Architecture B vs C vs A)
- "Your call" as a way to avoid owning the technical decision
- Asking for direction at every uncertain moment instead of investigating + deciding
- Treating the user as a peer reviewer of technical trade-offs

**When in doubt:** execute under the agreed principles. If the result is wrong, the user will correct it — they have repeatedly demonstrated they will. Manufacturing decision points to get pre-approval is its own form of drift; it doubles their cognitive load without changing the outcome.

**2026-06-02 RE-AFFIRMATION (after over-correcting):** following the `.7.2.10`/`.7.2.11` regression I caused, I over-corrected into asking the user to pick the next task even for SAFE, read-only, obviously-next work (e.g. "shall I run the read-only gap-report diagnostic next?"). User pushed back: *"do you really need me to pick the next task to work on?"* — No. The correct lesson from the regression is **be more rigorous and fact-driven (tools-first, one change at a time, measure), NOT ask permission for low-risk steps.** Just PNT into the obvious next read-only/analysis/leaf-owned work under the standing autonomous mandate. Reserve questions for GENUINE forks: a real direction choice with materially different outcomes, or anything with regression/release/external side-effects (the finite ask-list above). Read-only diagnostics, investigations, tool-builds, and leaf-owned no-behavior-change work are MINE to start without asking. Asking for direction at every clean checkpoint is the same drift this memory was written to stop — the post-regression scare is not a reason to amplify it.

**Cross-references:**
- [[feedback_no_workarounds_fix_hierarchy]] — the agreed fix hierarchy (the principle I should be deciding under, not asking about)
- [[feedback_tools_first_no_guessing]] — the agreed diagnostic approach (decide what tool to run; don't ask which)
- [[feedback_prefer_grammar_leave_engine_alone]] — strong default I should be applying without asking
- [[feedback_push_pacing]] — concrete instance of "this DOES require explicit per-instance authorization"
- [[feedback_hook_scope]] — concrete instance of "settings.json DOES require explicit user authorization"
