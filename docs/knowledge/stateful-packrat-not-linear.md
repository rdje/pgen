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

**✅ EMPIRICALLY CONFIRMED on PGEN/SV (`PARSE-TERMINATION.1`, 2026-06-03).** A
complexity-scaling probe (`parseability_probe --parse systemverilog`, N typedef+use pairs
vs N wires, N ∈ {100..3200}): **stateful store-gated parsing scales ~N^1.66 with a
per-doubling ratio that GROWS toward 4 (2.45→2.89→3.24→3.58→3.84) — trending QUADRATIC**
(0.11 s → 34.8 s for 100→3200); the **stateless baseline is ~N^1.08 = linear** (0.04 s →
1.70 s). So PGEN's stateful packrat is *not* linear — confirmed, not assumed. Likely the
cause of historical uvm_pkg parse slowness.

**ROOT CAUSE PINNED (`PARSE-TERMINATION.3`, 2026-06-03 — profiled; corrects the "conditional
memoization" guess):** it is NOT the memo (keyed (rule,pos), never cleared) and NOT has_fact
(indexed `by_kind`, O(1)). A macOS `sample` profile of the store_3200 parse shows the hot path
is **`with_semantic_runtime_rule_transaction` cloning the ENTIRE `SemanticRuntimeState`** per
rule transaction (`std::mem::take` + `.clone()`, `ast_based_generator.rs:1281-1283`) + the
matching `drop_in_place` — O(state) per call × O(rule-calls) = O(N²). **FIX (efficient
snapshot/restore, Laurent & Mens SLE 2016):** swap the clone-snapshot/clone-restore for the
existing, proven-complete `checkpoint()` + `rollback_to_named()` (O(changes)) — already used in
the try_parse path. Implementation = `PARSE-TERMINATION.3.1` (codegen swap + regen 10 parsers +
verify). Expected: store-gated parsing N^1.66 → ~N^1.0 (linear).
