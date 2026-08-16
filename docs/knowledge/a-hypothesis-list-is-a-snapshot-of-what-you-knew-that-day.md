---
id: a-hypothesis-list-is-a-snapshot-of-what-you-knew-that-day
title: An open leaf's hypothesis list is dated evidence — when the project later learns a new mechanism, nothing goes back and re-reads it
answers:
  - "I opened a leaf with two hypotheses and the answer turned out to be neither — where did the process fail"
  - "we discovered a measurement trap; what else in the backlog is now wrong"
  - "how stale is an unworked task leaf"
  - "is it worth re-reading a queued leaf before working it"
  - "why did a wrong diagnosis survive three sessions in a tracked system"
  - "what should happen to open work items when a new trap is documented"
tags: [task-trees, evidence, measurement, process, continuity, routing]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.19 slice 1, 2026-08-16 session #241. `.19` was opened in session #232 because a re-derived SystemVerilog parser came out 99 747 bytes from the tracked artifact; it named two hypotheses (the flip's diff perturbed codegen / the git-ignored input JSON moved) and priced the measurements that would separate them. Both were wrong. The real mechanism — a generated parser embeds its own `-o` path once per rule-entry site, so 33 249 sites × 3 characters of path difference = 99 747 bytes, exactly — was discovered by the SAME TREE six sessions later, in session #238 (`.20` slice 3), where it inverted that slice's first reading before a normalisation step was added. Nothing connected the two. The leaf sat at the head of the queue for three further sessions carrying hypotheses the project already had the evidence to refute, and the refutation cost 1 m 44 s once it was attempted.
reverify: "bash docs/tasks/artifacts/engine_universal_services/es19_path_embedding/probe.sh   # exit 0; ARM 2 re-derives the 99 747 and ARM 3 proves the path is the ONLY difference"
---

**A hypothesis list is not a plan — it is a *measurement*, taken on the day it was written, of what
the project knew.** Task-tree systems are careful about statuses, frontiers and acceptance criteria
going stale. They are careless about hypotheses, because a hypothesis reads like an opinion, and
opinions do not feel like they can rot.

They can, and the rot is silent in the worst direction: **a wrong hypothesis list makes a leaf look
*more* ready to work, not less.** It has a named next step and a priced experiment. Nothing about it
signals "the answer may already be on disk".

## The shape

```text
session #232   leaf .19 opened   "99 747 bytes unaccounted for"
                                 hypothesis (a) the diff moved codegen
                                 hypothesis (b) the input moved
                                 ⇒ both plausible, both priced, neither eliminated

session #238   leaf .20 slice 3  discovers: a generated parser embeds its -o path
                                 36 346 times ⇒ path length changes artifact size
                                 (it had just INVERTED that slice's own reading)

session #239…  .19 sits at the head of the queue, unchanged

session #241   .19 slice 1       33 249 sites × 3 chars = 99 747, exactly.
                                 Neither hypothesis. 1 m 44 s to refute.
```

The gap is not three sessions of neglect. `.20` slice 3 recorded its discovery properly — it opened
`ENGINE-UNIVERSAL-SERVICES.25` to own the mechanism, wrote the normalisation into two instruments,
and stated the hazard. What it did not do, and what no doctrine asked it to do, is **ask which
already-open leaves the new mechanism explains.**

## Why the obvious remedies do not work

- ❌ *"Re-read every open leaf when you learn something."* The backlog is dozens of leaves; this is a
  cost with no trigger, so it becomes a suggestion, and a rule nothing checks is a suggestion.
- ❌ *"The new leaf should link the old one."* `.25` could not have known `.19` was an instance —
  that is exactly the inference that was missing.
- ❌ *"Work leaves in order and it comes out in the wash."* It did come out in the wash. It cost
  three sessions of a leaf sitting at the head of the queue describing an investigation that was
  already answered.

## What actually helps

**Invert the direction: the trigger belongs on the leaf being WORKED, not on the leaf being
written.** Before executing a queued leaf's priced experiment, spend one minute asking *what has the
project learned since this was written?* — `git log` between the leaf's opening commit and HEAD,
scoped to the same tree, is a one-command answer:

```bash
git log --oneline <leaf-opening-commit>..HEAD -- docs/tasks/<TREE>.md
```

That is cheap, it has a trigger (you are about to work the leaf), and it is scoped to the tree most
likely to have learned the relevant thing — measured here, the mechanism that refuted `.19` came
from `.19`'s own tree.

⭐ **And the cheapest hypothesis is worth testing before the expensive one, even when it is not on
the list.** `.19`'s two priced experiments both needed a build at an old commit. The hypothesis that
turned out to be correct needed one `grep -oF … | wc -l` against an artifact already on disk. When a
priced experiment is the *first* thing you reach for, the list has stopped being a list of
hypotheses and started being a plan.

## The same class, one abstraction up: an OPTION list

⭐ A leaf that proposes fixes ages exactly like a leaf that proposes causes, and it is easier to
miss because an option list reads like a decision already half-made.

Measured two days later in the same tree (`ENGINE-UNIVERSAL-SERVICES.24`, 2026-08-16): a leaf offered
two ways to close a gap — hash the probe binary, or emit a fingerprint into the generated parser —
and argued for the second. Both were worse than a third nobody had written down: compute the
fingerprint at **build** time rather than at **emit** time. The hook that makes it work (`build.rs`
already resolving the artifact's path and already declaring `rerun-if-changed` on it) had been
sitting there the whole time, and choosing it costs **zero** generated bytes where the recommended
option would have moved every artifact in the tree.

The same leaf also carried a *reason* worth re-deriving: it rejected the first option because the
binary is *"rebuilt often, so identity would churn on every rebuild"*. Measured over 200 commits, the
churn floor is **5** — the premise was right in direction and wrong in magnitude, and magnitude was
what the choice turned on.

⇒ two questions before executing any option a leaf recommends: **what is not on this list?** and
**does the reason it gives for rejecting the others still measure true?**

## The honest bound

This card documents a class, not a gate. There is no mechanical check here, deliberately: detecting
"an open leaf's hypotheses are now refutable" requires knowing the answer, which is the work itself.
What is transferable is the **question**, asked at the one moment it is cheap — the minute before
you start the priced experiment.

Related: [[a-measurement-that-cannot-name-its-instrument-cannot-be-checked-for-staleness]],
[[an-ab-whose-arms-are-secretly-identical-does-not-fail-it-passes]],
[[a-conservation-control-cannot-catch-a-misassignment]].
