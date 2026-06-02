<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_drift_status_aligned.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: drift_status must be literal "aligned" when shapes match
description: The ast_shape_contract test enforces `drift_status == "aligned"` for any sample where current_content_kind matches expected_content_kind. Using `calibrated_<date>_slice_N` (or any non-aligned string) for a matching sample fails the gate. Slice provenance belongs in calibration_history, not in drift_status.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
When updating a per-grammar ast-shape-contract manifest (e.g. `rust/test_data/ast_shape_contract/systemverilog_v1.json`) during slice work:

**Why:** the test `ast_shape_contract::tests::<grammar>_ast_shape_contract_holds_against_running_generated_parser` calls `aligned_assertion_failures` whenever a sample's runtime content_kind matches the manifest's expected content_kind. In that path it strictly requires `drift_status == "aligned"`. Any other string (even `"calibrated_2026_05_14_slice_115"`) panics with `current_content_kind matches expected_content_kind but drift_status is "..." (must be "aligned")`.

**How to apply:**
- For samples whose shape MATCHES the manifest (the common case for new slice work that doesn't touch the sample): leave `drift_status: "aligned"`.
- For samples whose shape DOES drift: use a descriptive string like `calibrated_<date>_slice_N` or `rule_level_annotation_not_applied_for_multibranch_or_root` AND populate `drift_tracked_in` with the explanation.
- Log slice provenance in the manifest's top-level `calibration_history` array (each entry one-line, prepended at the front), not by bumping drift_status.
- The `drift_status` field is for manifest-vs-runtime alignment, not slice version tracking.

Burned by this between SV-Slice-101 and SV-Slice-115: each slice bumped drift_status to `calibrated_<date>_slice_N` which silently failed the gated test until SV-Slice-116 corrected it back to `"aligned"`.
