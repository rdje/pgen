---
id: a-cap-with-no-headroom-is-a-cap-about-to-be-raised
title: A size cap reports compliance, not health — watch HEADROOM, and remember a two-axis cap binds only on its tighter axis
answers:
  - "my file is passing its size cap — is that enough to leave it alone"
  - "the pre-commit hook just blocked me on a size cap — should I raise it"
  - "how much headroom should a capped file have"
  - "I have both a line cap and a byte cap — do I need to watch both"
  - "why did my line cap never fire even though the file kept growing"
  - "what should I move out of the resume pointer when it gets too big"
  - "how do I demote content out of a capped file without losing the pointers"
  - "how do I prove a demotion lost nothing"
tags: [memory-architecture, caps, continuity, instrument-honesty, layering]
date: 2026-08-09
status: current
evidence: docs/tasks/MEMORY-ARCH.md leaf .6 (7 156 -> 4 961 B of a 7 168 B cap; 12 -> 2 207 B spare; 38 -> 29 of 50 lines); docs/decisions/project_north_star.md + docs/decisions/project_standing_tripwires.md (the two demotion destinations); MEMORY_ARCHITECTURE.md §6 (the 60-line / 138 403-byte measurement that created the second axis); scripts/check_memory_architecture.sh (both caps + the index<->record sync)
reverify: "b=$(wc -c < MEMORY.md); echo \"$b of ${MEMORY_POINTER_BYTE_CAP:-7168} bytes\"; test \"$b\" -lt 6100 && echo HEADROOM-OK"
---

**A cap answers "am I compliant?", which is binary and lagging. The question that predicts the next
failure is "how much room is left?"** Measured: a bounded resume pointer sat at **7 156 of 7 168
bytes** — passing, and 12 bytes from blocking. Every routine update to that file is 300–500 bytes,
so the *next* commit would have hit the hook. And the moment a cap blocks you is the worst possible
moment to decide policy about it: the work is already done, the fix that unblocks you in one line is
raising the number, and the doctrine that forbids exactly that
(`MEMORY_ARCHITECTURE.md` §6 — *"never raise a cap to fit the content; that is the failure restated
as a policy"*) is competing with the urge to land finished work.

⇒ **Treat headroom as the metric.** A capped file wants to sit at roughly two-thirds full, so that
ordinary updates never approach the edge and the decision to demote is made calmly, in its own
change, rather than under commit pressure.

⛔ **A two-axis cap binds only on its tighter axis — and can silently become one-axis.** The same
file had a 50-line cap and a 7 168-byte cap. It was at **38 of 50 lines**: the line cap had no
prospect of ever firing, because the growth was arriving as *longer lines*, not more of them. Two
caps existed precisely so neither wrapped prose nor very long lines could bypass the budget — but
once one axis is slack, the design silently degrades to the single-axis form it was built to
replace. **Check which axis is actually binding; if one has not moved in months, it is decoration.**

⭐ **Demotion is not a consolation prize — done properly it beats the state it replaces.** Content
gets demoted because its *lifecycle* is wrong for the layer, not because it is unimportant. Here two
blocks were moved out: a curated constraints list and a standing-traps list, together 45 % of the
cap, both with append-once/supersede lifecycles in an overwrite-only layer. Moving them to
addressable records made them **retrievable by question for the first time** — in the resume pointer
they were reachable only by reading the resume pointer.

⚠️ **Prove the demotion lost nothing, mechanically.** The risk is a dropped pointer, and prose review
will not catch one. Extract every link and identifier from the *pre-change* file and assert each is
still reachable from {the new file} ∪ {the destination records}:

```bash
# 19/19 wiki-links and 23/23 leaf ids still reachable, 0 lost
git show HEAD:MEMORY.md > /tmp/old.md   # then diff the extracted link sets, not the prose
```

That check takes a minute, runs from git, and converts "I think I kept everything" into a number.
