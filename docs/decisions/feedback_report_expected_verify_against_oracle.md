<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_report_expected_verify_against_oracle.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_report_expected_verify_against_oracle
description: "A downstream bug report's stated \"expected\" is itself fix-adjacent — verify it against the independent executable spec oracle, which can overturn BOTH your candidate fix and the report's claim; and trace a PEG fix through every ordered-choice fallback incl. catch-alls."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

When fixing a downstream-reported parser bug, the report's
`expected_behavior` / stated literal is **not** an oracle — it is
another fix-adjacent artifact (often derived from a stale tool
version, wrong mode, or the reporter's own model). If an authoritative
**executable** spec oracle is available, run it for the exact matrix
and derive expecteds from *it*, independent of both the fix and the
report. This is the sharp corollary of
[[feedback_corpus_expected_from_spec_not_fix]]: there, a self-authored
probe was the mirror; here, the *report itself* is the mirror.

**Why:** RGX-0087 — the RGX report asserted `\89`@0-groups should
compile as the literal `"89"` (and an `#[ignore]`d RGX unit test
"asserts the correct PCRE2 behaviour"). Running the authoritative
oracle `pcre2test` 10.47 (the PCRE2 version family RGX vendors) over
the full 13-case matrix showed `\89`@0-groups is a **compile error
115**, NOT literal. The oracle simultaneously falsified (a) the
task-file's candidate fix and (b) the report's stated expected. Baking
either into the grammar would have shipped a spec-wrong fix and a
spec-wrong consumer note.

**How to apply:** before designing a parser fix from a downstream
report — (1) locate the authoritative executable oracle (`pcre2test`,
reference impl, conformance corpus `testinput/testoutput`) and run the
*exact* reproducer + family matrix through it; treat its verdict as
the target, explicitly noting where it contradicts the report (put the
corrected expectation in the ledger/contract/book consumer-note so the
downstream doesn't restore a spec-wrong expectation). (2) Trace the
candidate fix through **every** PEG ordered-choice fallback, including
catch-all rules: RGX-0087's single negative-lookahead was insufficient
because `simple_escape = !"o{" … any_char` is a catch-all that would
still consume `\8` as a shorthand after `backreference` failed — the
correct fix needed *two* guards (the failing production *and* the
downstream catch-all). A one-production analysis is not a fix proof in
a backtracking grammar. Extra rigor when the bug is a regression from
your own prior fix: probe the whole matrix before *and* after, prove
the prior fix's cases byte-identical.
