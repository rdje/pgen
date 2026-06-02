<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_codegen_outer_branch_remap.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Codegen outer-branch remap fix (2026-05-14)
description: extract_rule_annotations() in ast_pipeline/mod.rs counts every `|` as a branch_idx but the AST after step2_group_by_or only has top-level branches — without the outer-branch remap, parse_rule_content's truncate(branch_count) silently drops annotations on patterns like `digit ( sep | digit )*`, `( a | b )? id`, and per-branch annotations on branches that contain inner-Or groups.
type: project
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
Fix landed 2026-05-14 in `rust/src/ast_pipeline/mod.rs` (extract_rule_annotations) and mirrored in `rust/src/ast_shape_contract.rs` (extract_declared_annotations_from_json crosscheck). Adds `outer_branch_idx` and `branch_to_outer` tracking so annotations land on the OUTER (top-level) branch index rather than the inner-counted index that was getting truncated.

**Why:** parser-codegen silently dropped `return_object` annotations for any rule whose body had inner-`|` (parens-grouped Or). Discovered while trying SV slices 107/112/113 — `binary_value`/`unsigned_number` (Pattern A: `digit ( sep | digit )*`), `ps_type_identifier_sv_2017/2023` (Pattern B: `( a | b | c )? id`), and individual branches like `ansi_port_declaration.named_dot` and `method_call_receiver_*.hierarchical` (Pattern C: branch with `( a | b )? id` shape inside a multi-branch rule).

**How to apply:** the fix is now baseline behavior; new grammar work doesn't need workarounds. The previously-deferred patterns (slices 107/112/113 had documented "DROPPED" annotations) can be revisited and re-annotated cleanly. The SV typing campaign's natural endpoint reached at slice 113 (2256 annotations) recovered +27 annotations on the rerun (now 2283) from previously-suppressed inner-branch entries.

Side effects to know:
- `return_annotation_v1.json` manifest had a stale duplicate `string_literal` branch=1 entry (artifact of pre-fix broadcast-then-truncate). Cleaned up at fix time.
- `arrow_then_dollar1` sample in same manifest still has drift_status `rule_level_annotation_not_applied_for_multibranch_or_root` — that's a SEPARATE codegen issue (rule-level annotation at end of multi-branch rule needs to broadcast to all top-level branches, currently lands only on the current branch_idx). Fix is orthogonal.
