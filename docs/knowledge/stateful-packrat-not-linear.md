---
id: stateful-packrat-not-linear
title: WARNING — PGEN's stateful packrat is NOT guaranteed linear (semantic store = state)
answers:
  - "is PGEN's parser guaranteed linear time / hang-free"
  - "does packrat memoization guarantee no catastrophic backtracking here"
  - "why might the SV parser hang or go exponential"
  - "what is the fix for stateful packrat non-linearity"
  - "does the semantic store affect parse-time complexity"
tags: [parser, termination, hang, packrat, warning, literature]
date: 2026-06-03
status: current
evidence: "Ford Packrat Parsing ICFP 2002 (linear ONLY for stateless PEG); Chida & Kawakoya, Is Stateful Packrat Parsing Really Linear in Practice?, CC 2020 (counter-example + conditional-memoization fix, 260x/217x)"
reverify: see docs/tasks/PARSE-TERMINATION.md; grep -n "memoiz" rust/src/ast_pipeline/*.rs
---

**Do not assume PGEN parses in linear time / cannot hang.** Ford's packrat linear-time
guarantee (*Packrat Parsing*, ICFP 2002) holds **only for a STATELESS PEG**. PGEN's
**semantic store makes it a STATEFUL PEG**, and **Chida & Kawakoya, *Is Stateful Packrat
Parsing Really Linear in Practice?* (CC 2020)** proves real-world stateful grammars can run
**exponentially** (memoization keyed only on position becomes unsound when results depend on
mutable global state).

**The fix (adopt, don't invent):** the same paper proposes a *PEG with variable bindings* +
**stateful packrat with conditional memoization** — memoize only the state relevant to
global-state use, not the full state — giving **260× / 217×** time/space improvement on
pathological inputs while restoring polynomial time.

**Implication for PGEN:** Pillar B (`PARSE-TERMINATION`) must FIRST *measure* whether our
stateful packrat is actually linear (a complexity-scaling probe) before claiming hang-free,
and adopt conditional memoization if it isn't. This also sharpens the already-backlogged
PARSE-SOTA "memo-soundness audit." Owned by `PARSE-TERMINATION.1`/`.3`. Related:
[[try_parse_must_snapshot_semantic_state]], [[feedback_question_bypasses_manual_cleanup]].
