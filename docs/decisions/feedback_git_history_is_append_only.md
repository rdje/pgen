# Git history is append-only — never amend, rebase, or rewrite; correct with a NEW commit

**Category:** `feedback` (standing discipline)
**Stated:** 2026-07-29 (session #223), director, in response to a report that mentioned rewriting
history as an option that had been considered and declined.

## The directive (verbatim)

> *"Do you mean git history? Why do you want to rewrite history? I am not asking for that. People
> shall be able to see the whole commit history. That history shall not be hacked, amended or
> anything. Create a new commit to change something you wrote in a previous one."*

⭐ **"People", not "I"** (director's own clarification, same session). The audience is **anyone who
ever reads this repository** — a downstream consumer, a future maintainer, an auditor, another
harness — not just the director reviewing today. That widens the rule's force: history is a public
record, and rewriting it removes evidence from readers who were never asked.

## The rule

**The commit history is an append-only record.** It is the director's view of what actually
happened, and it must remain complete and unaltered.

- ⛔ **Never** `git commit --amend`, `git rebase` (interactive or otherwise), `git reset --hard` over
  committed work, `git push --force`, or any other operation that rewrites, drops or edits an
  existing commit — **including one that is only local and unpushed.** "Unpushed" is not a licence:
  the history is the record either way.
- ✅ **To correct anything in an earlier commit — message, content, or a mistaken claim — make a NEW
  commit that states the correction.** The original stays visible; the correction sits after it.
- ⚠️ Do not ASK for permission to rewrite either. The answer is settled, and raising it invites the
  misreading that a rewrite was wanted. Just make the corrective commit.

## Why

A history that can be edited is not a record — it is a draft. **People** read this project through
its commits: the director reviewing today, a downstream consumer auditing what changed before a
release, a future maintainer reconstructing why, another agent resuming after a lost session. A
rewritten commit removes evidence from every one of them, and silently changes what a
previously-reported SHA refers to — including SHAs already cited in task-trees, `CHANGES.md` and
prior reports. This is the same principle the project already
applies to its documents: **corrections are made forward and left visible, never back-dated**
(*"the finding stays visible, not back-dated"* — `CI-PARITY-GATE-ROT.15`, `DESIGN-PRIOR-ART`'s
retro-fitted PRIOR ART note, and `DONE-BAR.1`'s refutation of its own tree's first-pass reading).
This record extends that discipline from the docs to the git layer, where it is even stronger:
docs get amended in place by design; history never does.

## The occasion (recorded so the rule has a worked example)

Commit **`94454e30`** (leaf `DONE-BAR.1`) was labelled `PGEN-DONE-BAR-0002` — a slice id already
consumed by **`598038a8`** (session #221). The collision was corrected **forward**: `docs/tasks/DONE-BAR.md`
records it, the `.1` slice is referred to by its SHA, and the session's later slices resume at
`0005`. Nothing was amended.

⚠️ **A gap this exposed, recorded not mechanized:** nothing checks slice-id uniqueness.
`.githooks/commit-msg` requires an identifier-shaped work-unit id in the subject and never asks
whether that id is already taken; a one-line structural check over `git log` would have blocked it.
Deliberately **not** built at one known occurrence — `GENERATED-LINT-CORRECTNESS.4`'s pricing rule
says price a candidate against the whole corpus before adopting it, and one occurrence is below that
bar. Revisit on the second.

## Related

- `COMMIT.md` — "Pre-Commit Safety Rules": *do not use destructive git commands unless explicitly
  requested.* This record **strengthens** that: for history rewriting there is no "unless" — it is
  not to be requested or offered.
- [[feedback_always_signoff_decisions]] · [[feedback_instrument_needs_ground_truth]]
