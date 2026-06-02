<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_corpus_expected_from_spec_not_fix.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-corpus-expected-from-spec-not-fix
description: When building your own verification corpus/probe, derive each expected value from the authoritative spec — never from what your fix is intended to do. Otherwise the corpus rubber-stamps your bug (false pass), and only a separate no-regression gate catches it.
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Lesson (RGX-0084, 2026-05-18):** I built a `\NN` family probe whose
expected column I derived from *my fix's intended behavior* (gate
every numeric backref on group count). PCRE2's actual rule is
different (single digit `\1`–`\9` is **always** a backref; only `N≥10`
is gated). So my probe reported "ALL GREEN" while the fix was wrong
for every single-digit-insufficient-groups case — a **false pass**,
because the oracle and the implementation shared the same error. It
was caught only by an independent existing regression test
(`regex_parser_pgen_rgx_0081…`, which correctly expected `\1`→numeric)
failing in the non-negotiable no-regression run.

**Why:** a self-authored corpus whose expectations encode "what I
think my change should do" is not an oracle — it is a mirror. It
cannot detect a wrong premise; it amplifies it into confident
"verified" claims.

**How to apply:**
- Derive every expected value in a verification corpus from the
  **authoritative source** (the spec / report / `pcre2pattern(3)` /
  reference oracle), transcribed *before* and *independently* of
  designing the fix — not from the fix's intended behavior.
- Treat a 100%-green self-authored probe as *necessary, not
  sufficient*. The real proof is the independent no-regression gate
  (existing pinned tests) staying green. Never skip it, never
  conclude "fixed" on the self-probe alone.
- If a long-standing pinned regression test "fails because of my
  change", first ask "is the test encoding correct behavior that my
  change wrongly broke?" before assuming the test is stale. Here it
  was correct and I was wrong.

Related: [[feedback_ebnf_consult_annotation_docs]] (verify the
mechanism via the real artifact, not by inspection),
[[feedback_grammar_edit_proof_gate_lockstep]] (a grammar/engine edit
owns ALL its downstream proof gates same-slice — the no-regression
gate is what saved this).
