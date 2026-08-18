---
id: a-control-that-names-its-subject-by-revision-expires
title: A RED control that identifies its subject by REVISION expires the moment the fix lands — identify it by the PROPERTY that makes it the subject, and it keeps working forever
answers:
  - "how do I write a RED control that proves my guard changed something"
  - "my before/after control uses HEAD~1 — is that safe once I commit"
  - "how do I prove a refusal is the guard and not just that nothing was going to happen"
  - "how should a probe fetch the pre-fix version of the code it is testing"
  - "why did my adversarial arm start passing vacuously after the fix landed"
  - "what makes a control durable rather than true-on-the-day"
tags: [controls, ground-truth, instruments, red-arm, durability, git, evidence]
date: 2026-08-17
status: current
evidence: SV-CORPUS-GRAD.13c.2f(d). A guard was added to `tools/extract_systemverilog_lrm_profiles.py` so it refuses to overwrite an output it did not reproduce. Ten green refusal arms proved nothing on their own — each is equally consistent with *the guard works* and *nothing was going to be written here*. The RED arm runs the PREVIOUS revision of the tool at the same copy of the deliverable and measures `rc=0` with return annotations `2292 -> 0`, i.e. a silent destruction of the AST contract. The first draft fetched that revision with `git show HEAD~1:<tool>`; the moment the guard is committed, HEAD carries the guard, `HEAD~1` drifts to an unrelated commit, and the arm would test the guarded version and pass vacuously. Rewritten to walk `git rev-list -n 40 HEAD -- <tool>` and take the first revision whose source does NOT contain the refusal string — probe `EXTRACTOR-GUARD 14/14`, arm A13 reporting the revision it selected.
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/es13c2f4d_extractor_guard/probe.sh 2>&1 | tail -3   # A13 must name a revision, A14 must go RED"
---

**A control is a claim about a difference.** Naming its subject by *where it sits in history* makes
the claim true only while history stands still — and the act of landing the fix is exactly what moves
history.

## The failure shape

```bash
# ❌ expires on the commit that makes the control necessary
git show HEAD~1:tools/thing.py > /tmp/before.py   # today: the unguarded version
                                                  # after commit: the guarded version, one older
```

The arm still runs. It still prints a row. It just tests the wrong thing, and it does so **in the
passing direction** — the guarded version refuses, the arm sees a refusal, and a suite that was built
to detect a missing guard now reports that the guard is present because it tested the guard.

## The fix: select by the property, not the position

```bash
# ✅ "the newest revision that does NOT have the property under test"
prev=""
for rev in $(git rev-list -n 40 HEAD -- "$TOOL"); do
  if ! git show "$rev:$TOOL" | grep -q "REFUSING to overwrite"; then prev="$rev"; break; fi
done
[ -n "$prev" ] || { echo "cannot prove the guard changed anything"; exit 1; }
```

Two things this buys that the revision form cannot:

1. **It survives every future commit.** Ten more edits to the tool, and the arm still finds the last
   version without the guard.
2. **It fails loudly when it cannot do its job.** If no unguarded revision exists in range — the
   guard was there from the start, or the search window is too short — the probe says so instead of
   silently testing nothing. Print the selected revision (`A13 … (b8a5f954)`) so a reader can see
   *which* subject was measured.

## The companion rule

⛔ **An arm that only ever runs against a copy proves the guard fires on copies.** Run the decisive
arms against the **real** paths too, and assert the real file's `sha256` is unchanged afterwards —
that is the property that actually matters, and a scratch-only test cannot state it. The safety net
is version control: the target must be committed before the arm runs.

## Where else this applies

Any before/after control whose "before" is fetched from history: a retired enforcer re-executed to
show it passed on the old input, a pre-fix classifier replayed to show it misses rows the new one
catches, a stale-baseline arm. Each of them is one commit away from testing itself.
See also [[a-conservation-control-cannot-catch-a-misassignment]] — a control can be green because it
is blind, or because it is aimed at the wrong subject; these are different failures with the same
symptom.
