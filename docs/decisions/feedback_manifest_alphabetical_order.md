<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_manifest_alphabetical_order.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: AST shape-contract manifest is alphabetical
description: When adding entries to rust/test_data/ast_shape_contract/regex_v1.json, insert each at its lexicographic slot by rule name — not by semantic family.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
The `regex_v1.json` shape-contract manifest is **alphabetically ordered by rule name**. The grammar emits return-annotation entries alphabetically (NOT in source-file order, NOT by semantic family). The contract test validates manifest entry [N] against grammar entry [N]; any ordering mismatch fails.

When a slice adds multiple manifest entries, traverse the rules alphabetically and insert each at its lexicographic slot:

- `atomic_group` goes BEFORE `backreference` (a < b)
- `capturing_group` goes between `branch_reset_group` and `comment_group` (br < ca < co)
- `noncapturing_group` goes between `name_ref` and `octal_escape` (na < no < oc)

**Why:** Discovered during regex.ebnf slice 21 (simple groups). First insertion attempt grouped 4 new entries (atomic_group, capturing_group, noncapturing_group, branch_reset_group) by semantic family between `branch_reset_group` and `comment_group`. Contract test failed with many `manifest[N] != grammar[N]` mismatches because the grammar emits these at their alphabetical positions, scattered across the manifest. Re-inserting each at its alphabetical slot fixed all mismatches.

**How to apply:** Before inserting manifest entries, run `grep -n '"rule":' rust/test_data/ast_shape_contract/regex_v1.json` to see the existing alphabetical sequence; then traverse and insert each new entry at its correct slot. Don't try to put related entries together for "readability" — the grammar's alphabetical emit order is authoritative.
