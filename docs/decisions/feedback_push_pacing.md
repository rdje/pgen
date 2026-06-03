<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_push_pacing.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: push-pacing-push-is-release-pace-it
description: "Pushing = releasing. Push every 100 commits OR explicit user \"push now\"; do NOT ask for / suggest push while unpushed < 100. Updated 2026-06-03 (cadence RAISED ~30 -> 100); supersedes the 2026-05-26 ~30 cadence."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

**⛔ ACTIVE RULE (user, 2026-06-03 — RAISES the cadence, supersedes ~30):**
> "push shall occur every 100 commits or whenever I explicitly ask for it." + "do not ask for push unless there are at least 100 commits."

(Prior rule, user 2026-05-26 — superseded by the 100-commit cadence above:)
> "Please, stop pushing after everything PNT loop. Pushing is releasing. Wait at least around 30 commits or unless I explicitly ask for it."

**Why:** push to `origin/main` IS the release surface. Every push lands on remote, where downstream consumers (RGX, anything else) can pull. Pacing pushes protects the user's review window — they want to look at the accumulated slice queue before any of it goes live.

**How to apply:**
- Commit per leaf/slice freely (commits are local checkpoints — commit aggressively).
- After each commit, do NOT run `git push`.
- Track unpushed-commit count vs `origin/main` silently as work progresses.
- Push only when EITHER:
  - The unpushed count reaches **100** (the batch boundary, user 2026-06-03), OR
  - The user explicitly says "push" / "push now" / "ship it" / similar.
- **Do NOT even ASK about / suggest pushing while unpushed < 100** (user 2026-06-03, emphatic). Just keep committing.
- If the user explicitly says "hold" / "don't push yet" / similar, that pauses even the 100-batch trigger until lifted.
- A one-time "push" grant pushes the current accumulation only; do NOT generalize it into a standing per-slice push policy.
- After a push, the count resets — the cadence resumes from there.

**This session's anti-pattern (the trigger for this rule update, 2026-05-26):**
- User said "Please push whenever you can" — I pushed 61 commits (correct).
- After that, I committed Slice-79 (`.37.3`) and pushed it solo (1 commit, premature).
- Then committed Slice-80 (`.37.4`) and pushed it solo too (1 commit, premature).
- The user observed "Pushing is releasing" — i.e. each of those was a small, unbatched release.

**Lesson:** "push when you can" is not "push after every PNT loop iteration." It is "the accumulated batch is ready for release now." Subsequent slices accumulate again until the next batch is ready (~30 commits) or the user asks.

**Historical context (now consolidated):**
- 2026-05-12: PNT-loop mode — don't pause to ask, just keep committing.
- 2026-05-14: one-time push grants are one-time.
- 2026-05-17: explicit "hold" supersedes the 30-cap.
- 2026-05-18: "push every 30" was the default cadence.
- 2026-05-19: absolute no-push override (suspended the 30-cap; required explicit per-push auth).
- 2026-05-26: ~30-commit cadence OR explicit user "push now".
- 2026-06-03 (CURRENT): cadence RAISED to **100 commits** OR explicit ask; do NOT ask for / suggest push while unpushed < 100. Explicit hold still supersedes.

**Decision tree:**
- About to push? Did the user explicitly say push? → push.
- Have 100 unpushed commits accumulated AND no explicit hold? → push.
- Otherwise → commit, do NOT push.

**Restore tag:** `checkpoint/sv-exh-proof-3.2-clean` @ `41bef35e` (still valid).
