---
id: a-status-that-is-prose-is-a-status-nothing-checks
title: A work item whose status lives in free prose can be FIXED and still read `todo` — because nothing derives the status from what the item itself recorded
answers:
  - "my tracker says a task is open but the fix already shipped — how does that happen"
  - "how do I stop a task board from over-reporting remaining work"
  - "the commit that fixed it named the ticket and the ticket still says todo"
  - "is routing a defect to another owner enough to keep the original item current"
  - "why does remaining-work always look bigger than it is"
  - "what should a status field be derived from"
  - "how do I audit whether my open items are actually still open"
tags: [tracking, task-trees, process, evidence, staleness, governance]
date: 2026-08-23
status: current
evidence: "SV-CORPUS-GRAD.13c.2b (PGEN-SV-CORPUS-GRAD-0279) and .13c.2d (PGEN-SV-CORPUS-GRAD-0280), found in one session. .13c.2b read `blocked on ENGINE-UNIVERSAL-SERVICES.13` for 9 days after ENGINE-UNIVERSAL-SERVICES.17 slice 9 fixed it in another tree. .13c.2d read `todo` for 6 days after its own tree's slice 3 fixed it in commit 6d1a18b3, whose subject reads verbatim `leaf SV-CORPUS-GRAD.13c.2d fix SHIPPED`. Both statuses live as free prose inside a Markdown heading; no instrument compares that prose against the leaf's own recorded slices or against the tracked reproducer manifest, whose `class` column already reads `fixed` for both. A first-cut census was attempted and its result deliberately withheld: it matched several headings per leaf and returned 6 suspects of 11, the heading-grammar trap that DOCTRINE-GAP-OWNERSHIP.8 owns."
reverify: "grep -c 'fixed' stimuli/sv/adjudication_repros/MANIFEST.tsv; git log -1 --format=%s 6d1a18b3   # the manifest already knows which defects are fixed, and the fixing commit already named its leaf — two oracles the status prose was never compared against"
---

**A status field is a claim about the world, and like any claim it is worth what the thing that
checks it is worth.** When the status is free prose — a word in a heading, a label typed once when
the item was opened — nothing checks it at all. It is not merely *unverified*; it is *unverifiable*,
because there is no producer to re-derive it from.

The failure is quiet and it has a direction:

- an item that is **done but reads open** inflates remaining work,
- so the plan looks *bigger* than it is,
- so nobody investigates, because over-reporting never triggers an alarm.

Under-reporting gets caught the moment someone ships against it. Over-reporting can persist for the
life of a project.

## Two mechanisms, and only one of them is the obvious one

**Cross-owner drift.** Item A is blocked on a defect owned by item B in another area. B is fixed and
closed. A keeps saying `blocked` — because the routing left a pointer from A to B and none from B
back to A. This is the diagnosis everyone reaches for, and it is real.

⭐ **It is not sufficient.** The second observed case had *no* routing involved: the item was fixed
by its own team, in its own tree, by a commit whose subject line **named the item**. The status
still did not move. So the cause is not a missing back-pointer between owners. It is that **the
status is a separate artifact from the work**, written once, never re-derived — and separate
artifacts drift.

## What to derive it from

Two shapes, and they trade off differently:

1. **From the item's own record.** If closing an item means recording a slice that says `CLOSES`,
   the status is a function of the item's contents and a checker is a few lines. ⚠️ The catch is
   that recording work is not the same as finishing it, so this needs an explicit closing marker —
   otherwise "has slices" gets read as "is done", which is the same error in the other direction.
2. **From an external oracle that already knows.** Very often one exists and nobody looked. In the
   founding case a tracked reproducer manifest carried a `class` column literally reading `fixed`
   for both stale items, and a 2.25-second suite re-derived it from the shipped artifact. An oracle
   with an independent producer is the stronger arm, because it cannot agree with the status by
   construction.

⛔ Whichever is chosen, the check needs a **red arm** — an item whose status is knowingly stale,
kept so the checker is proven able to say so. A status checker that has only ever been green is
itself a status in prose.

## Do not publish the census before the census is sound

The instinct on finding two instances is to sweep for the rest and report a number. Resist it until
the sweep can be trusted. The first attempt here matched *several headings per item* — the item's
own, its sub-items', and any heading that merely mentioned it — so most "suspects" carried
contradictory labels from different lines, and the count was meaningless. Publishing it would have
replaced an honest *"two confirmed, rate unknown"* with a confident wrong number.

⇒ **Two hand-verified instances with dates are a finding. A number from an instrument you have not
validated is not.** Report the first, withhold the second, and name what the census is blocked on.

⚠️ **Bound.** Both instances come from one tree and both were found incidentally, while doing the
work rather than while auditing it — so the *rate* is unmeasured and could genuinely be two. What is
established is the mechanism, not its frequency.
