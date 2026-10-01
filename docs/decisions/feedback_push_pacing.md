<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_push_pacing.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: push-pacing-push-is-release-pace-it
description: "Pushing = releasing. Push every 400 commits OR on the director's explicit EXCEPTIONAL order — an ordered push does NOT license follow-up autonomous pushes; do NOT ask for / suggest push while unpushed < 400. Updated 2026-10-01 (cadence RAISED 300 -> 400, logged as an ADR on the director's order); supersedes the 2026-07-13 300-commit cadence, whose ordered-push-is-one-time correction stands."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

**⛔ ACTIVE RULE (director, 2026-10-01 — RAISES the cadence, supersedes 300; every other clause of the 2026-07-13 rule stands):**
> "So, please set the official push cadence to 400. And log it as an ADR"

**Context (2026-10-01).** The director asked "what is the push cadence?" while
`git rev-list --count origin/main..HEAD` printed 316 — past the 300 trigger, reached at `b69eceb39` —
and the pre-push full CI mirror (directive 16, `ci_workflow_local_gate`) had not yet gone green on that
accumulation. The director raised the cadence rather than order an exceptional push.

**Decision.** The official push cadence is **400 commits** unpushed against `origin/main`, or the
director's explicit EXCEPTIONAL order.

**Consequences.**
- A count from 300 to 399 is no longer a push trigger: at 316 the push is NOT due. The next trigger is
  400 unpushed, or an explicit order.
- The full CI mirror still runs before every push (directive 16, unchanged), so it now fires at the 400
  boundary. It remains the only instrument that walks the cold clone, the surface audits and the
  no-feature test posture, so a red of those classes can live up to 400 commits rather than 300. The
  director's open spend call on a more frequent cold chain (the 2026-09-30 callout record, item 3) is
  not settled by this ruling.
- "Do NOT ask about / suggest pushing" now applies below 400.
- Live copies of the number moved in the same commit (`PGEN-DOCS-PUSH-PACING-0002`): `MEMORY.md`, this
  record's `INDEX.md` row, `docs/book/src/gate-flow.md`, and three knowledge cards. Dated task-leaf
  text, `CHANGES.md` entries and the 2026-09-30 callout that say 300 are history and stay as written.
  Two comment copies in code-classified files — `rust/scripts/ci_workflow_local_gate.sh` and
  `.github/workflows/lib-unit-test-gate.yml` — still read 300; each rides the next change to its file.

(Prior rule, director 2026-07-13 — its 300-commit cadence is superseded by 400 above; its
one-time-order correction STANDS:)
> "The last push was exceptional. The push you just did was really not necessary. From now on the push
> cadence shall be every 300 commits or unless I exceptionally say so."

**The 2026-07-13 anti-pattern (the trigger):** the director ordered a push (exceptional, correct to execute);
LATER the same session I pushed a small follow-up batch autonomously "for a clean handoff before /exit" —
WRONG: an ordered push is a one-time grant, and a handoff/exit is NOT a push trigger (commits are already
durable locally; the remote cadence is the director's release rhythm).

(Prior rule, user 2026-06-07 — superseded by the 300-commit cadence above:)
> "Push cadence is everything 200 commits or until I explicitly ask for it."

(Prior rule, user 2026-06-03 — superseded by the 200-commit cadence above:)
> "push shall occur every 100 commits or whenever I explicitly ask for it." + "do not ask for push unless there are at least 100 commits."

**Why:** push to `origin/main` IS the release surface. Every push lands on remote, where downstream consumers (RGX, anything else) can pull. Pacing pushes protects the user's review window — they want to look at the accumulated slice queue before any of it goes live.

**How to apply:**
- Commit per leaf/slice freely (commits are local checkpoints — commit aggressively).
- After each commit, do NOT run `git push`.
- Track unpushed-commit count vs `origin/main` silently as work progresses.
- Push only when EITHER:
  - The unpushed count reaches **400** (the batch boundary, director 2026-10-01), OR
  - The director explicitly and EXCEPTIONALLY says "push" / "push now" / "ship it" / similar.
- **Do NOT even ASK about / suggest pushing while unpushed < 400.** Just keep committing.
- If the director explicitly says "hold" / "don't push yet" / similar, that pauses even the 400-batch trigger until lifted.
- A one-time "push" grant pushes the current accumulation only; do NOT generalize it into a standing per-slice push policy,
  and it does NOT license any later autonomous push the same session (2026-07-13 correction). A handoff / session end /
  "safe to /clear" is NOT a push trigger — local commits are already the durable layer.
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
- 2026-06-03: cadence RAISED to **100 commits** OR explicit ask; do NOT ask for / suggest push while unpushed < 100. Explicit hold still supersedes.
- 2026-06-07: cadence RAISED to **200 commits** OR explicit ask; do NOT ask for / suggest push while unpushed < 200. Explicit hold still supersedes.
- 2026-07-13: cadence RAISED to **300 commits** OR the director's explicit EXCEPTIONAL order; an ordered push is one-time (no autonomous follow-ups; handoff/exit is not a trigger).
- 2026-10-01 (CURRENT): cadence RAISED to **400 commits** OR the director's explicit EXCEPTIONAL order; the one-time-order rule and the no-ask-below-the-cadence rule carry over unchanged.

**Decision tree:**
- About to push? Did the director explicitly order THIS push? → push (that grant is now consumed).
- Have 400 unpushed commits accumulated AND no explicit hold? → push.
- Otherwise → commit, do NOT push. (Handoff-ready ≠ push.)

**Restore tag:** `checkpoint/sv-exh-proof-3.2-clean` @ `41bef35e` (still valid).
