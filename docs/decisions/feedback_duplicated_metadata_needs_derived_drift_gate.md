<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_duplicated_metadata_needs_derived_drift_gate.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-duplicated-metadata-needs-derived-drift-gate
description: A version/identity value duplicated across surfaces (Rust const, JSON manifest, ledger, book, contract) WILL drift. The durable fix is a spec-derived drift gate that parses the single authoritative source and asserts the copies equal it — not another hardcoded pin (which is just one more copy that drifts).
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Lesson (PGEN-RGX-0086, 2026-05-18):** the regex release/contract
version lived in *three* runtime copies — `REGEX_PARSER_RELEASE_VERSION`
/`REGEX_PARSER_INTEGRATION_CONTRACT_VERSION` consts AND
`regex_parser_integration_contract_v1.json` — plus the authoritative
`PGEN_RELEASED_PARSER_BUG_LEDGER.md` "Fixed in" labels (and the
book/contract prose). The consts drifted ~46 minors behind the ledger
because nothing tied them together; ~ten releases bumped the ledger
and never touched the consts. A pre-existing `*_metadata_is_stable`
test "pinned" the values — but it pinned the **stale** ones, so it
*encoded the defect* and silently rode along with the drift.

**Why:** every hardcoded duplicate of an authoritative value is a
future drift site. A "stability" test that asserts duplicate ==
duplicate (const == JSON) catches *inconsistency between copies* but
NOT *collective drift from the source of truth* — they can all be
wrong together. Only a check anchored to the single authoritative
source closes it.

**How to apply:**
- For any value duplicated across surfaces, add a **spec-derived
  drift gate**: a test that *parses the single authoritative source*
  (here: the ledger's table-row "Fixed in" cells, max-by-release) and
  asserts every copy equals it. Do NOT "fix" drift by hardcoding the
  current value in the test — that is just another copy that will
  drift; the gate must re-derive from the source each run.
- When you bump one copy, grep for ALL copies (a failing
  consistency test often reveals a 2nd/3rd you didn't know about —
  here the JSON mirror surfaced via `_is_stable`). Prefer a single
  source of truth (have the JSON/contract reference the const, or all
  reference the ledger) over N hand-synced copies.
- A long-standing pin/"stable" test failing on your correctness fix:
  first ask "is it pinning the defect?" ([[feedback_corpus_expected_from_spec_not_fix]]).
  Here it was — corrected to the spec value, not reverted.
- Derive the target from the spec/authoritative artifact, never from
  the downstream report's literal (the report's `1.1.75` was captured
  against an older pin; the ledger-latest was `1.1.77/1.1.79`).

Related: [[feedback_corpus_expected_from_spec_not_fix]] (expecteds
from the authoritative source; test-vs-change discrimination),
[[feedback_grammar_edit_proof_gate_lockstep]] (a change owns ALL its
downstream proof/metadata surfaces same-slice).
