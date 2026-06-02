<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/reference_sv_defect_taxonomy.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: reference_sv_defect_taxonomy
description: "Living tracking document for every systematic defect class found in the SV-EXH-PROOF campaign — categories A–G, fields per class, audit checklist, amendment log"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

`docs/reference/SV_EXH_PROOF_DEFECT_TAXONOMY.md` — the single source of truth for systematic defect classes uncovered in the SV-EXH-PROOF campaign (and reusable for VHDL/RTL when those campaigns hit similar patterns). Categories: **A** LRM-extraction defects, **B** PEG semantics, **C** codegen, **D** grammar semantic-annotation, **E** engine/resolver gaps, **F** tooling/diagnostic gaps, **G** design risks. Each class has fields: ID, Status (✅/🟡/🔴/⚠/🛠/🩹), Discovered (leaf + commit), Pattern, Root cause, Instances, Fix, **Audit pattern** (the mechanical grep / oracle that finds new instances), Related links.

**When to amend (binding):** any new systematic class encountered → add an entry. Class fix landed → update Status + fill Fix + add Amendment-log row. Audit-pattern sweep finds more instances → inline or split. The audit-pattern entries are the "we don't miss anything" mechanism — they are the durable contribution of each entry.

Linked from `docs/tasks/SV-EXH-PROOF.md` header. Cross-references [[feedback_verify_sv_parser_regen_mtime]], [[feedback_question_bypasses_manual_cleanup]], [[feedback_layer_0_unified_quantifier]], [[feedback_semantic_annotation_no_dotted_refs]], [[project_b61_producer_pass_plan]].
